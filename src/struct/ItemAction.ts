import type ActionParams from "./ActionParams.ts";
import type Field from "./Field.ts";

export default class ItemAction {
    public readonly name: string;
    private readonly params: ActionParams;
    private readonly _url: string;

    constructor(name: string, url: string, params: ActionParams) {
        this.name = name;
        this.params = params;
        this._url = url;
    }

    public url(item: object): string {
        let url = this._url;

        this.params.params.forEach((field: Field) => {
            url = url.replace(`:${field.name}`, field.get_from_item(item).toString());
        });

        return `#${url}`;
    }
};
