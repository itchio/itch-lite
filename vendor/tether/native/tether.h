#ifndef TETHER_H
#define TETHER_H

/* Generated with cbindgen:0.9.1 */

/* This file is autogenerated. You probably don't want to modify it by hand. */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * A reference to a window.
 */
typedef struct {
    uint8_t _unused[0];
} _tether_dummy;

/**
 * Pointer type for tether windows
 */
typedef struct _tether *tether;

typedef struct {
    /**
     * The HTTP status code for the response
     */
    uintptr_t status_code;
    /**
     * The contents of the response.
     */
    const uint8_t *content;
    /**
     * Length of the contents of the response (in bytes).
     */
    uintptr_t content_length;
} tether_net_response;

/**
 * A network request
 */
typedef struct {
    /**
     * The URI that has been requested
     */
    const char *request_uri;
    /**
     * Closure context for respond
     */
    const void *respond_ctx;
    /**
     * What to respond with, if 'response_set' is true
     */
    void (*respond)(const void *ctx, const tether_net_response *res);
} tether_net_request;

/**
 * Configuration options for a window.
 */
typedef struct {
    /**
     * Initial width of the window in pixels (TODO: figure out HIDPI).
     */
    uintptr_t initial_width;
    /**
     * Initial height of the window in pixels (TODO: figure out HIDPI).
     */
    uintptr_t initial_height;
    /**
     * Width below which the window cannot be resized.
     */
    uintptr_t minimum_width;
    /**
     * Height below which the window cannot be resized.
     */
    uintptr_t minimum_height;
    /**
     * When set, don't show OS decorations.
     */
    bool borderless;
    /**
     * When set, enable debug interface, for example Microsoft Edge DevTools Preview
     */
    bool debug;
    /**
     * The data to pass to event handlers.
     */
    void *data;
    /**
     * The window received a message via `window.tether(string)`.
     */
    void (*message)(void *data, const char *message);
    /**
     * The window was closed, and its resources have all been released.
     */
    void (*closed)(void *data);
    /**
     * A network request was made
     */
    void (*net_request)(void *data, tether_net_request *req);
} tether_options;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Close the window.
 */
void tether_close(tether self_);

/**
 * Schedule a function to be called on the main thread.
 *
 * All the `tether` functions should only be called on the main thread.
 */
void tether_dispatch(void *data, void (*func)(void *data));

/**
 * Run the given script.
 */
void tether_eval(tether self_, const char *js);

/**
 * Stop the main loop as gracefully as possible.
 */
void tether_exit(void);

/**
 * Focus the window, and move it in front of the other windows.
 *
 * This function will not steal the focus from other applications.
 */
void tether_focus(tether self_);

/**
 * Display the given HTML.
 */
void tether_load(tether self_, const char *html);

/**
 * Open a new window with the given options.
 */
tether tether_new(tether_options opts);

/**
 * Start the main loop and call the given function.
 *
 * This function should be called on the main thread, and at most once. It
 * should be called before any other `tether` function is called.
 */
void tether_start(void (*func)(void));

/**
 * Set the window's title.
 */
void tether_title(tether self_, const char *title);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* TETHER_H */
