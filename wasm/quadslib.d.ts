/* tslint:disable */
/* eslint-disable */
/**
*/
export class ImageApproximation {
  free(): void;
/**
* @param {RGBAImage} im
* @param {number} max_iter
* @returns {ImageApproximation}
*/
  static constructor(im: RGBAImage, max_iter: number): ImageApproximation;
/**
* @returns {number}
*/
  im_result_data_as_pointer(): number;
/**
* @returns {boolean}
*/
  next(): boolean;
/**
* @returns {number}
*/
  current_iter: number;
/**
* @returns {number}
*/
  error: number;
/**
* @returns {number}
*/
  max_iter: number;
}
/**
*/
export class RGBAImage {
  free(): void;
/**
* @param {Uint8Array} data
* @param {number} width
* @param {number} height
* @returns {RGBAImage}
*/
  static constructor(data: Uint8Array, width: number, height: number): RGBAImage;
/**
* @returns {number}
*/
  data_as_pointer(): number;
/**
* @returns {number}
*/
  width(): number;
/**
* @returns {number}
*/
  height(): number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_rgbaimage_free: (a: number) => void;
  readonly rgbaimage_constructor: (a: number, b: number, c: number, d: number) => number;
  readonly rgbaimage_data_as_pointer: (a: number) => number;
  readonly rgbaimage_width: (a: number) => number;
  readonly rgbaimage_height: (a: number) => number;
  readonly __wbg_imageapproximation_free: (a: number) => void;
  readonly __wbg_get_imageapproximation_max_iter: (a: number) => number;
  readonly __wbg_set_imageapproximation_max_iter: (a: number, b: number) => void;
  readonly __wbg_get_imageapproximation_current_iter: (a: number) => number;
  readonly __wbg_set_imageapproximation_current_iter: (a: number, b: number) => void;
  readonly __wbg_get_imageapproximation_error: (a: number) => number;
  readonly __wbg_set_imageapproximation_error: (a: number, b: number) => void;
  readonly imageapproximation_constructor: (a: number, b: number) => number;
  readonly imageapproximation_im_result_data_as_pointer: (a: number) => number;
  readonly imageapproximation_next: (a: number) => number;
  readonly __wbindgen_malloc: (a: number) => number;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
        