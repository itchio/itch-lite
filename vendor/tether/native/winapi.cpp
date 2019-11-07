//TODO: Windows currently doesn't show any context menu.

#define WIN32_LEAN_AND_MEAN
#define UNICODE

#include <sdkddkver.h>
#include <objbase.h>
#include <Windows.h>
#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Web.UI.Interop.h>
#include <winrt/Windows.Web.Http.h>

#include "tether.h"

using namespace winrt;
using namespace Windows::Foundation;
using namespace Windows::Web::UI;
using namespace Windows::Web::UI::Interop;
using namespace Windows::Web::Http;

// ===============
// RANDOM NONSENSE
// ===============

const LPCWSTR WINDOW_CLASS = L"BORING";
const UINT WM_APP_DISPATCH = WM_APP;
static DWORD MAIN_THREAD;
static WebViewControlProcess WEBVIEWS { nullptr };

// Pump the main loop until the future has been resolved.
template <typename T> auto block(T const& async) {
    if (async.Status() != AsyncStatus::Completed) {
        handle h(CreateEvent(nullptr, false, false, nullptr));
        async.Completed([h = h.get()](auto, auto) { SetEvent(h); });
        HANDLE hs[] = { h.get() };
        DWORD i;
        CoWaitForMultipleHandles(
            COWAIT_DISPATCH_WINDOW_MESSAGES | COWAIT_DISPATCH_CALLS | COWAIT_INPUTAVAILABLE,
            INFINITE, 1, hs, &i
        );
    }
    return async.GetResults();
}

// Get the bounding rectangle of a window.
Rect getClientRect(HWND hwnd) {
    RECT clientRect;
    GetClientRect(hwnd, &clientRect);
    return Rect(
        (float) (clientRect.left),
        (float) (clientRect.top),
        (float) (clientRect.right - clientRect.left),
        (float) (clientRect.bottom - clientRect.top)
    );
}

// A closure to be run on the main thread.
struct Dispatch {
    void *data;
    void (*func)(void *data);
};

struct RespondCtx {
    const WebViewControlWebResourceRequestedEventArgs *args;
};

static void _tether_respond(const void *vctx, uintptr_t status_code, const char *content) {
    fprintf(stderr, "In _tether_respond...\n");
    auto ctx = (RespondCtx*) vctx;

    auto res = HttpResponseMessage();
    res.StatusCode(HttpStatusCode(status_code));
    res.Content(HttpStringContent(winrt::to_hstring(content)));
    ctx->args->Response(res);
    fprintf(stderr, "Leaving _tether_respond...\n");
}

struct _tether {
    HWND hwnd;
    WebViewControl webview = nullptr;
    tether_options opts;

    _tether(tether_options opts): opts(opts) {
        hwnd = CreateWindow(
            WINDOW_CLASS,
            L"",
            opts.borderless ? 0 : WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            (int) opts.initial_width,
            (int) opts.initial_height,
            nullptr,
            nullptr,
            GetModuleHandle(nullptr),
            nullptr
        );

        webview = block(WEBVIEWS.CreateWebViewControlAsync((int64_t)hwnd, getClientRect(hwnd)));

        SetWindowLongPtr(hwnd, GWLP_USERDATA, (LONG_PTR)this);

        webview.AddInitializeScript(L"window.tether = function (s) { window.external.notify(s); };");
        auto data = opts.data;
        auto message = opts.message;
        webview.ScriptNotify([=](auto const&, auto const& args) {
            std::string s = winrt::to_string(args.Value());
            message(data, s.c_str());
        });

        webview.ContentLoading([=](auto const &, auto const& args) {
            fprintf(stderr, "ContentLoading\n");
        });
        webview.DOMContentLoaded([=](auto const &, auto const& args) {
            fprintf(stderr, "DOMContentLoaded");
        });
        webview.FrameContentLoading([=](auto const &, auto const& args) {
            fprintf(stderr, "FrameContentLoading\n");
        });
        webview.FrameDOMContentLoaded([=](auto const &, auto const& args) {
            fprintf(stderr, "FrameDOMContentLoaded\n");
        });
        webview.FrameNavigationCompleted([=](auto const &, auto const& args) {
            fprintf(stderr, "FrameNavigationCompleted\n");
        });
        webview.FrameNavigationStarting([=](auto const &, auto const& args) {
            fprintf(stderr, "FrameNavigationStarting\n");
        });
        webview.LongRunningScriptDetected([=](auto const &, auto const& args) {
            fprintf(stderr, "LongRunningScriptDetected\n");
        });
        webview.NavigationCompleted([=](auto const &, auto const& args) {
            fprintf(stderr, "NavigationCompleted\n");
        });
        webview.NavigationStarting([=](auto const &, auto const& args) {
            fprintf(stderr, "NavigationStarted\n");
        });
        webview.NewWindowRequested([=](auto const &, auto const& args) {
            fprintf(stderr, "NewWindowRequested\n");
        });
        webview.PermissionRequested([=](auto const &, auto const& args) {
            fprintf(stderr, "PermissionRequested\n");
        });
        webview.UnsafeContentWarningDisplaying([=](auto const &, auto const& args) {
            fprintf(stderr, "UnsafeContentWarningDisplaying\n");
        });
        webview.UnviewableContentIdentified([=](auto const &, auto const& args) {
            fprintf(stderr, "UnviewableContentIdentified\n");
        });

        webview.UnsupportedUriSchemeIdentified([=](auto const&, auto const& args) {
            fprintf(stderr, "Unsupported Uri Scheme Identified!\n");
            fprintf(stderr, "It was %S\n", args.Uri().ToString().c_str());
            args.Handled(TRUE);
        });

        webview.WebResourceRequested([=](auto const&, auto const& args) {
            auto uri = winrt::to_string(args.Request().RequestUri().ToString());

            RespondCtx ctx;
            ctx.args = &args;

            tether_net_request net_req;
            net_req.request_url = uri.c_str();
            net_req.respond_ctx = &ctx;
            net_req.respond = _tether_respond;

            fprintf(stderr, "Invoking rust...\n");
            opts.net_request(&net_req);
            fprintf(stderr, "Back from rust...\n");
        });

		bool saved_fullscreen = false;
		RECT saved_rect;
		LONG saved_style = -1;
		webview.ContainsFullScreenElementChanged([=](auto const &sender, auto const &) mutable {
			bool fullscreen = sender.ContainsFullScreenElement();
			if (fullscreen == saved_fullscreen) return;
			saved_fullscreen = fullscreen;

			if (sender.ContainsFullScreenElement()) {
				// Save the window position and size and stuff so we can restore it later.
				GetWindowRect(hwnd, &saved_rect);
				saved_style = GetWindowLong(hwnd, GWL_STYLE);
				// Enter fullscreen mode.
				SetWindowLong(hwnd, GWL_STYLE, saved_style & ~(WS_CAPTION | WS_THICKFRAME));
				MONITORINFO mi;
				mi.cbSize = sizeof mi;
				GetMonitorInfo(MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST), &mi);
				RECT screen_rect = mi.rcMonitor;
				SetWindowPos(
					hwnd,
					HWND_TOP,
					screen_rect.left,
					screen_rect.top,
					screen_rect.right - screen_rect.left,
					screen_rect.bottom - screen_rect.top,
					SWP_FRAMECHANGED
				);
			} else {
				// Exit fullscreen mode, restoring the window's properties.
				SetWindowLong(hwnd, GWL_STYLE, saved_style);
				SetWindowPos(
					hwnd,
					HWND_TOP,
					saved_rect.left,
					saved_rect.top,
					saved_rect.right - saved_rect.left,
					saved_rect.bottom - saved_rect.top,
					SWP_FRAMECHANGED
				);
			}
		});

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    }
};

// The window's event handler.
static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    tether window = (tether)GetWindowLongPtr(hwnd, GWLP_USERDATA);

    switch (msg) {
        case WM_CLOSE:
            DestroyWindow(hwnd);
            break;
        case WM_DESTROY:
            (window->opts.closed)(window->opts.data);
            delete window;
            break;
        case WM_SIZE:
            window->webview.Bounds(getClientRect(hwnd));
            break;
        case WM_GETMINMAXINFO:
            if (window) {
                LPMINMAXINFO lpMMI = (LPMINMAXINFO)lParam;
                lpMMI->ptMinTrackSize.x = (LONG) window->opts.minimum_width;
                lpMMI->ptMinTrackSize.y = (LONG) window->opts.minimum_height;
                break;
            }
        default:
            return DefWindowProc(hwnd, msg, wParam, lParam);
    }

    return 0;
}

LPWSTR to_wide(LPCSTR input) {
    auto input_len = strlen(input);

    auto codepage = CP_UTF8;
    DWORD flags = 0;

    // N.B: this is in wchars, not bytes
    auto output_len = MultiByteToWideChar(
        codepage, flags,
        input, input_len,
        NULL, 0
    );

    auto output = (LPWSTR) calloc(sizeof(WCHAR), output_len);
    MultiByteToWideChar(
        codepage, flags,
        input, input_len,
        output, output_len
    );
    return output;
}

// ==============
// EXPORTED STUFF
// ==============

void tether_start(void (*func)(void)) {
    winrt::init_apartment(winrt::apartment_type::single_threaded);

    HINSTANCE hi = GetModuleHandle(nullptr);
    MAIN_THREAD = GetCurrentThreadId();
    WEBVIEWS = WebViewControlProcess();

    WNDCLASSEX cls;
    cls.cbSize = sizeof cls;
    cls.style = CS_HREDRAW | CS_VREDRAW;
    cls.lpfnWndProc = WndProc;
    cls.cbClsExtra = 0;
    cls.cbWndExtra = 0;
    cls.hInstance = hi;
    cls.hIcon = LoadIcon(nullptr, IDI_APPLICATION);
    cls.hCursor = LoadCursor(nullptr, IDC_ARROW);
    cls.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);
    cls.lpszMenuName = nullptr;
    cls.lpszClassName = WINDOW_CLASS;
    cls.hIconSm = nullptr;
    RegisterClassEx(&cls);

    func();

    MSG msg;
    BOOL res;
    while ((res = GetMessage(&msg, nullptr, 0, 0))) {
        if (res == -1) break;

        if (msg.hwnd) {
            TranslateMessage(&msg);
            DispatchMessage(&msg);
            continue;
        }

        Dispatch *dispatch;
        switch (msg.message) {
            case WM_APP_DISPATCH:
                dispatch = (Dispatch *)msg.lParam;
                dispatch->func(dispatch->data);
                delete dispatch;
                break;
        }
    }
}

void tether_dispatch(void *data, void (*func)(void *data)) {
    PostThreadMessage(
        MAIN_THREAD,
        WM_APP_DISPATCH,
        0,
        (LPARAM)new Dispatch({ data, func })
    );
}

void tether_exit(void) {
    PostQuitMessage(0);
}

tether tether_new(tether_options opts) {
    return new _tether(opts);
}

void tether_eval(tether self, const char *js) {
    self->webview.InvokeScriptAsync(
        L"eval",
        single_threaded_vector<hstring>({ winrt::to_hstring(js) })
    );
}

void tether_load(tether self, const char *html) {
    self->webview.NavigateToString(winrt::to_hstring(html));
}

void tether_title(tether self, const char *title) {
    auto w_title = to_wide(title);
    SetWindowText(self->hwnd, w_title);
    free(w_title);
}

void tether_focus(tether self) {
    SetActiveWindow(self->hwnd);
}

void tether_close(tether self) {
    PostMessage(self->hwnd, WM_CLOSE, 0, 0);
}

void *tether_alloc(uintptr_t size) {
    return malloc(size);
}
