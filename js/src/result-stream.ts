/**
 * [GPT-5] sq-7lk — a browser-safe RDF/JS query `ResultStream` over a pull iterator.
 *
 * The RDF/JS type extends Node's `EventEmitter`, but importing `node:events` at runtime would
 * break the package's browser build. This class implements the interoperable event subset used
 * by the Query and Stream specifications (`on` / `once` / `removeListener` / `off` / `emit`),
 * plus Node-style `pause` / `resume` controls for backpressure.
 */
import type * as RDF from '@rdfjs/types';

type EventName = string | symbol;
type StreamListener = ((...args: unknown[]) => void) & { listener?: StreamListener };

/** RDF/JS `ResultStream` with the pull backpressure controls this implementation exposes. */
export interface SparqResultStream<T> extends RDF.ResultStream<T> {
  pause(): this;
  resume(): this;
  destroy(error?: Error): this;
}

export class ResultStream<T> {
  readonly #source: () => Iterator<T>;
  readonly #listeners = new Map<EventName, StreamListener[]>();
  #iterator: Iterator<T> | undefined;
  #maxListeners = 10;
  #flowing = false;
  #pumpScheduled = false;
  #completionScheduled = false;
  #terminal = false;

  constructor(source: () => Iterator<T>) {
    this.#source = source;
  }

  /** Pulls one result, or `null` after exhaustion/error as required by RDF/JS. */
  read(): T | null {
    if (this.#terminal) return null;
    try {
      const next = this.#next();
      if (next.done) {
        this.#scheduleEnd();
        return null;
      }
      return next.value;
    } catch (error) {
      this.#scheduleError(error);
      return null;
    }
  }

  on(event: EventName, listener: StreamListener): this {
    let listeners = this.#listeners.get(event);
    if (!listeners) this.#listeners.set(event, (listeners = []));
    listeners.push(listener);
    // Like a Node readable, adding a data listener enters flowing mode. Pulling starts in a
    // later microtask so chained end/error listeners are installed before the first result.
    if (event === 'data') this.resume();
    return this;
  }

  addListener(event: EventName, listener: StreamListener): this {
    return this.on(event, listener);
  }

  once(event: EventName, listener: StreamListener): this {
    const wrapper: StreamListener = (...args) => {
      this.removeListener(event, wrapper);
      listener(...args);
    };
    wrapper.listener = listener;
    return this.on(event, wrapper);
  }

  removeListener(event: EventName, listener: StreamListener): this {
    const listeners = this.#listeners.get(event);
    if (!listeners) return this;
    for (let i = listeners.length - 1; i >= 0; i--) {
      const current = listeners[i]!;
      if (current === listener || current.listener === listener) {
        listeners.splice(i, 1);
        break;
      }
    }
    if (listeners.length === 0) this.#listeners.delete(event);
    return this;
  }

  off(event: EventName, listener: StreamListener): this {
    return this.removeListener(event, listener);
  }

  prependListener(event: EventName, listener: StreamListener): this {
    let listeners = this.#listeners.get(event);
    if (!listeners) this.#listeners.set(event, (listeners = []));
    listeners.unshift(listener);
    if (event === 'data') this.resume();
    return this;
  }

  prependOnceListener(event: EventName, listener: StreamListener): this {
    const wrapper: StreamListener = (...args) => {
      this.removeListener(event, wrapper);
      listener(...args);
    };
    wrapper.listener = listener;
    return this.prependListener(event, wrapper);
  }

  removeAllListeners(event?: EventName): this {
    if (event === undefined) this.#listeners.clear();
    else this.#listeners.delete(event);
    return this;
  }

  listeners(event: EventName): StreamListener[] {
    return (this.#listeners.get(event) ?? []).map((listener) => listener.listener ?? listener);
  }

  rawListeners(event: EventName): StreamListener[] {
    return [...(this.#listeners.get(event) ?? [])];
  }

  listenerCount(event: EventName, listener?: StreamListener): number {
    const listeners = this.#listeners.get(event) ?? [];
    if (!listener) return listeners.length;
    return listeners.filter((current) => current === listener || current.listener === listener).length;
  }

  eventNames(): EventName[] {
    return [...this.#listeners.keys()];
  }

  setMaxListeners(count: number): this {
    if (!Number.isInteger(count) || count < 0) throw new RangeError('max listeners must be a non-negative integer');
    this.#maxListeners = count;
    return this;
  }

  getMaxListeners(): number {
    return this.#maxListeners;
  }

  emit(event: EventName, ...args: unknown[]): boolean {
    const listeners = this.#listeners.get(event);
    if (!listeners || listeners.length === 0) {
      if (event === 'error') {
        const error = args[0];
        throw error instanceof Error ? error : new Error(`Unhandled ResultStream error: ${String(error)}`);
      }
      return false;
    }
    // Snapshot like EventEmitter: listener changes during an emission affect later events only.
    for (const listener of [...listeners]) listener(...args);
    return true;
  }

  /** Stops cursor advancement after the currently emitted result. */
  pause(): this {
    this.#flowing = false;
    return this;
  }

  /** Continues emitting one result per microtask until paused or exhausted. */
  resume(): this {
    if (!this.#terminal) {
      this.#flowing = true;
      this.#schedulePump();
    }
    return this;
  }

  /** Closes an abandoned cursor immediately, then emits `end` (or the supplied `error`). */
  destroy(error?: Error): this {
    if (this.#terminal) return this;
    try {
      this.#iterator?.return?.();
    } catch (closeError) {
      this.#fail(closeError);
      return this;
    }
    if (error) this.#fail(error);
    else this.#end();
    return this;
  }

  #next(): IteratorResult<T> {
    this.#iterator ??= this.#source();
    return this.#iterator.next();
  }

  #schedulePump(): void {
    if (!this.#flowing || this.#terminal || this.#pumpScheduled) return;
    this.#pumpScheduled = true;
    queueMicrotask(() => {
      this.#pumpScheduled = false;
      if (!this.#flowing || this.#terminal) return;

      let next: IteratorResult<T>;
      try {
        next = this.#next();
      } catch (error) {
        this.#fail(error);
        return;
      }

      if (next.done) {
        this.#end();
        return;
      }

      this.emit('data', next.value);
      this.#schedulePump();
    });
  }

  #scheduleEnd(): void {
    if (this.#terminal || this.#completionScheduled) return;
    this.#completionScheduled = true;
    queueMicrotask(() => {
      this.#completionScheduled = false;
      this.#end();
    });
  }

  #scheduleError(error: unknown): void {
    if (this.#terminal || this.#completionScheduled) return;
    this.#completionScheduled = true;
    queueMicrotask(() => {
      this.#completionScheduled = false;
      this.#fail(error);
    });
  }

  #end(): void {
    if (this.#terminal) return;
    this.#terminal = true;
    this.#flowing = false;
    this.emit('end');
  }

  #fail(error: unknown): void {
    if (this.#terminal) return;
    this.#terminal = true;
    this.#flowing = false;
    this.emit('error', error instanceof Error ? error : new Error(String(error)));
  }
}

/**
 * The implementation deliberately remains browser-safe instead of subclassing Node's
 * `EventEmitter`; this bridge exposes its spec-complete runtime subset as the nominal RDF/JS
 * `ResultStream` type (the same pattern used by the package's quad stream adapter).
 */
export function asResultStream<T>(stream: ResultStream<T>): SparqResultStream<T> {
  return stream as unknown as SparqResultStream<T>;
}
