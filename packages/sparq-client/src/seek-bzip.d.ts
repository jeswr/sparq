declare module "seek-bzip" {
  interface SeekBzip {
    decode(bytes: Uint8Array): Uint8Array;
  }

  const codec: SeekBzip;
  export default codec;
}
