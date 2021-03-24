/* tslint:disable */
/* eslint-disable */
/**
* @param {string} json
* @param {number} target
* @returns {string | undefined}
*/
export function json2type(json: string, target: number): string | undefined;
/**
*/
export enum Target {
  Dataclass,
  DataclassWithJSON,
  PydanticBaseModel,
  PydanticDataclass,
  TypedDict,
  NestedTypedDict,
}
