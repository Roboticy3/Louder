// Scratch memory for calls that cross into WebAssembly.
//
// The C ABI has two shapes a binding has to cope with. An aggregate returned by
// value does not come back in a register: the caller hands over a pointer to
// space it owns and the engine writes there. An aggregate passed by value is
// copied into linear memory by the caller before the call. Both need memory
// this side of the boundary owns, and neither gets it from the engine.
//
// There is no allocator to call, because there is no glue code between here and
// the module. What there is instead is `memory.grow`, which hands out the
// pages above everything allocated so far and returns where they started. So
// the arena grows the memory and takes the pages it just got. Those pages are
// the engine's to give: the engine's own allocator asks for memory the same way
// and will be given what is above, never what is below. Nothing here is ever
// reused across a call, and nothing here is ever handed to the engine.
//
// A pointer stays valid for the life of the instance, because linear memory
// only ever grows. What does *not* survive a growth is the typed array over it:
// `ArrayBuffer` views detach when the buffer is replaced, so every accessor
// here re-reads `memory.buffer` rather than holding on to one.

const PAGE = 65536;

/**
 * Where each module's arena currently is.
 *
 * One per memory rather than one for the file, because a process may hold
 * several engines at once and each of them has its own linear memory. Sharing
 * one cursor between them hands the second engine a pointer into the first
 * one's address space, which is somewhere else entirely.
 */
const arenas = new WeakMap();

function arenaFor(memory) {
  let arena = arenas.get(memory);
  if (arena === undefined) {
    arena = { start: -1, end: -1 };
    arenas.set(memory, arena);
  }
  return arena;
}

function ceilPages(bytes) {
  return Math.max(1, Math.ceil(bytes / PAGE));
}

function align(bytes) {
  return (bytes + 7) & ~7;
}

/**
 * Take `bytes` of memory this side owns.
 *
 * @param {WebAssembly.Memory} memory
 * @param {number} bytes
 * @returns {number} a pointer into the module's linear memory
 */
export function reserve(memory, bytes) {
  const arena = arenaFor(memory);
  if (arena.start < 0 || arena.end - arena.start < bytes) {
    const before = memory.buffer.byteLength;
    memory.grow(ceilPages(Math.max(bytes, PAGE)));
    arena.start = before;
    arena.end = memory.buffer.byteLength;
  }
  const at = arena.start;
  arena.start += align(bytes);
  return at;
}

/** A `DataView` over the module's memory, current as of right now. */
export function view(memory) {
  return new DataView(memory.buffer);
}

/**
 * Copy out of the module's memory, so a value survives whatever the next call
 * does to it.
 *
 * Reading straight out of linear memory and handing the caller a view would
 * work until the next allocation, which is the kind of bug that shows up
 * somewhere else entirely. The copy is a few dozen bytes.
 */
export function take(memory, pointer, bytes) {
  const out = new Uint8Array(bytes);
  out.set(new Uint8Array(memory.buffer, pointer, bytes));
  return out;
}

/** Put a buffer of bytes into memory this side owns. */
export function put(memory, pointer, source) {
  new Uint8Array(memory.buffer).set(source, pointer);
}

const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8", { fatal: false });

/** UTF-8, which is the encoding the engine reads JSON in. */
export function toBytes(text) {
  return typeof text === "string" ? encoder.encode(text) : new Uint8Array(text);
}

export function fromBytes(bytes) {
  return decoder.decode(bytes);
}

/**
 * A buffer that the engine fills, growing until the whole answer fits.
 *
 * The engine truncates rather than refusing and reports how much fitted, so the
 * only way to be sure of the whole thing is to ask again with more room. Two
 * calls is the normal case; the loop is what makes a long document safe.
 */
export function readInto(memory, call, args, size = 4096) {
  for (;;) {
    const pointer = reserve(memory, size);
    const written = call(...args, pointer, size - 1);
    if (written < 0) {
      throw new RangeError("the engine reported " + written);
    }
    if (written < size - 1) {
      return take(memory, pointer, written);
    }
    size *= 4;
  }
}

/** Put text where the engine can read it, and hand over the pointer and length. */
export function withBytes(memory, text) {
  const bytes = toBytes(text);
  const pointer = reserve(memory, bytes.length + 1);
  put(memory, pointer, bytes);
  new Uint8Array(memory.buffer, pointer + bytes.length, 1)[0] = 0;
  return [pointer, bytes.length];
}
