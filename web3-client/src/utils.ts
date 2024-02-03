import { Hex } from "./hex";
import type { ObjectLike } from "./types";


export class Utils {
  static isObject(value: unknown): value is ObjectLike {
    return typeof value === "object" && value !== null;
  }

  static isArrayNonNullable<T = unknown>(value: unknown): value is Array<T> {
    return Array.isArray(value) && value.length > 0;
  }

  static hex2str(hex: string) {
    return Hex.hex2a(hex)
  }
}
