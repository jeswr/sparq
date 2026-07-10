/*! coi-serviceworker v0.1.7 - Guido Zuidhof and contributors, licensed under MIT */
// [OPUS-4.8] Vendored from https://github.com/gzuidhof/coi-serviceworker (MIT).
//
// WHY: GitHub Pages cannot set the COOP/COEP response headers that browsers require
// before they expose `SharedArrayBuffer` / set `window.crossOriginIsolated = true`.
// Without cross-origin isolation, `@aztec/bb.js` cannot spin worker threads, so the
// in-tab UltraHonk prover runs single-threaded (see src/lib/zk-prover.ts:maxThreads).
//
// This self-registering service worker re-serves every same-origin response with the
// isolation headers synthesised, so `crossOriginIsolated` becomes true on plain static
// hosting and bb.js can multithread (~4x faster proving on a multicore client).
//
// We use COEP `credentialless` (not `require-corp`) so cross-origin subresources that
// lack a CORP/CORS header still load — this keeps the rest of the static site working
// under isolation. The first load registers the SW and reloads once; thereafter the
// page is isolated. This changes NOTHING about what the ZK proof proves or its
// zero-knowledge property — it only unlocks the worker threads bb.js already supports.
//
// Served from /public, so under the `/sparq` basePath it lives at
// `/sparq/coi-serviceworker.js` with scope `/sparq/`, covering every app route.

let coepCredentialless = false;
if (typeof window === 'undefined') {
    self.addEventListener("install", () => self.skipWaiting());
    self.addEventListener("activate", (event) => event.waitUntil(self.clients.claim()));

    self.addEventListener("message", (ev) => {
        if (!ev.data) {
            return;
        } else if (ev.data.type === "deregister") {
            self.registration
                .unregister()
                .then(() => {
                    return self.clients.matchAll();
                })
                .then((clients) => {
                    clients.forEach((client) => client.navigate(client.url));
                });
        } else if (ev.data.type === "coepCredentialless") {
            coepCredentialless = ev.data.value;
        }
    });

    self.addEventListener("fetch", function (event) {
        const r = event.request;
        if (r.cache === "only-if-cached" && r.mode !== "same-origin") {
            return;
        }

        const request = (coepCredentialless && r.mode === "no-cors")
            ? new Request(r, {
                credentials: "omit",
            })
            : r;
        event.respondWith(
            fetch(request)
                .then((response) => {
                    if (response.status === 0) {
                        return response;
                    }

                    const newHeaders = new Headers(response.headers);
                    newHeaders.set("Cross-Origin-Embedder-Policy",
                        coepCredentialless ? "credentialless" : "require-corp"
                    );
                    if (!coepCredentialless) {
                        newHeaders.set("Cross-Origin-Resource-Policy", "cross-origin");
                    }
                    newHeaders.set("Cross-Origin-Opener-Policy", "same-origin");

                    return new Response(response.body, {
                        status: response.status,
                        statusText: response.statusText,
                        headers: newHeaders,
                    });
                })
                .catch((e) => console.error(e))
        );
    });

} else {
    (() => {
        const reloadedBySelf = window.sessionStorage.getItem("coiReloadedBySelf");
        window.sessionStorage.removeItem("coiReloadedBySelf");
        const coepDegrading = (reloadedBySelf == "coepdegrade");

        // You can customize the behavior of this script through a global `coi` variable.
        const coi = {
            shouldRegister: () => !reloadedBySelf,
            shouldDeregister: () => false,
            coepCredentialless: () => true,
            coepDegrade: () => true,
            doReload: () => window.location.reload(),
            quiet: false,
            ...window.coi
        };

        const n = navigator;
        const controlling = n.serviceWorker && n.serviceWorker.controller;

        // Record the failure if the page is served by serviceWorker.
        if (controlling && !window.crossOriginIsolated) {
            window.sessionStorage.setItem("coiCoepHasFailed", "true");
        }
        const coepHasFailed = window.sessionStorage.getItem("coiCoepHasFailed");

        if (controlling) {
            // Reload only on the first failure.
            const reloadToDegrade = coi.coepDegrade() && !(
                coepDegrading || window.crossOriginIsolated
            );
            n.serviceWorker.controller.postMessage({
                type: "coepCredentialless",
                value: (reloadToDegrade || coepHasFailed && coi.coepDegrade())
                    ? false
                    : coi.coepCredentialless(),
            });
            if (reloadToDegrade) {
                !coi.quiet && console.log("Reloading page to degrade COEP.");
                window.sessionStorage.setItem("coiReloadedBySelf", "coepdegrade");
                coi.doReload("coepdegrade");
            }

            if (coi.shouldDeregister()) {
                n.serviceWorker.controller.postMessage({ type: "deregister" });
            }
        }

        // If we're already coi: do nothing. Perhaps it's due to this script doing its job, or COOP/COEP are
        // already set from the origin server. Also if the browser has no notion of crossOriginIsolated, just give up here.
        if (window.crossOriginIsolated !== false || !coi.shouldRegister()) return;

        if (!window.isSecureContext) {
            !coi.quiet && console.log("COOP/COEP Service Worker not registered, a secure context is required.");
            return;
        }

        // In some environments (e.g. Firefox private mode) this won't be available
        if (!n.serviceWorker) {
            !coi.quiet && console.error("COOP/COEP Service Worker not registered, perhaps due to private mode.");
            return;
        }

        // Register the service worker from its own location so the scope matches the
        // basePath (e.g. /sparq/coi-serviceworker.js → scope /sparq/). [OPUS-4.8]
        // `document.currentScript.src` is the upstream path and resolves to the
        // basePath-prefixed URL when the browser runs this external script. Next.js
        // loads `beforeInteractive` scripts via its own bootstrap (`__next_s`); should
        // `currentScript` ever be null there, fall back to the basePath-prefixed
        // absolute path so the scope is still `/sparq/` (mirrors next.config basePath).
        const swPath =
            (document.currentScript && document.currentScript.src) ||
            (window.coiServiceWorkerPath || "/sparq/coi-serviceworker.js");
        n.serviceWorker.register(swPath).then(
            (registration) => {
                // [FABLE-5] sq-bx1zv — Firefox can FULFIL register() with `registration === undefined`
                // when service-worker registration is disallowed for the context (e.g. the Playwright
                // `serviceWorkers: "block"` setting, some private-browsing / policy configs). Chromium
                // rejects that case and takes the (err) branch below, but Firefox lands here with no
                // registration object, so the original `registration.scope` read threw
                // `TypeError: can't access property "scope", registration is undefined` — a real console
                // error surfaced in the nightly cross-browser (firefox) lane. Treat a missing
                // registration as a benign no-op: isolation headers simply are not applied (the same
                // graceful-degradation posture as the (err) reject branch), and the page keeps working
                // single-threaded. No `?.` — this vendored file must parse in the oldest SW-capable
                // engines, so an explicit guard is used.
                if (!registration) {
                    !coi.quiet && console.log(
                        "COOP/COEP Service Worker registration unavailable in this context; continuing without cross-origin isolation."
                    );
                    return;
                }
                !coi.quiet && console.log("COOP/COEP Service Worker registered", registration.scope);

                registration.addEventListener("updatefound", () => {
                    !coi.quiet && console.log("Reloading page to make use of updated COOP/COEP Service Worker.");
                    window.sessionStorage.setItem("coiReloadedBySelf", "updatefound");
                    coi.doReload();
                });

                // If the registration is active, but it's not controlling the page
                if (registration.active && !n.serviceWorker.controller) {
                    !coi.quiet && console.log("Reloading page to make use of COOP/COEP Service Worker.");
                    window.sessionStorage.setItem("coiReloadedBySelf", "notcontrolling");
                    coi.doReload();
                }
            },
            (err) => {
                !coi.quiet && console.error("COOP/COEP Service Worker failed to register:", err);
            }
        );
    })();
}
