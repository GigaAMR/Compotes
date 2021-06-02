import type Field from "./Field.ts";

export default class AssociatedItem<T> {
    public readonly item: T;
    public readonly fields: Field[];

    constructor(item: T, fields: Field[]) {
        this.item = item;
        this.fields = fields;
    }
}
