import type { Component } from 'solid-js';
import type { SetStoreFunction } from 'solid-js/store';
import SourceFileSelector from '../SourceFileSelector';

type ModelProps = {
    readonly ordinal: number;
    setModels: SetStoreFunction<ModelData[]>;
};

export type ModelData = {
    readonly ordinal: number;
    name: string;
    is_blank: boolean;
    model_source: string;
};

const Model: Component<ModelProps> = (props) => {
    const changeName = (newName: string) => {
        props.setModels(
            (model) => model.ordinal === props.ordinal,
            'name',
            () => newName,
        );
    };

    const changeIsBlank = (value: boolean) => {
        props.setModels(
            (model) => model.ordinal === props.ordinal,
            'is_blank',
            () => value,
        );
    };

    const changeSource = (value: string) => {
        props.setModels(
            (model) => model.ordinal === props.ordinal,
            'model_source',
            () => value,
        );
    };

    const removeModel = () => {
        props.setModels((models) => models.filter((model) => model.ordinal !== props.ordinal));
    };

    return (
        <li class="model">
            <label>
                Model Name:
                <input type="text" value="New Body Part" onChange={(event) => changeName(event.target.value)}></input>
            </label>
            <br />
            <label>
                Is Blank
                <input type="checkbox" onChange={(event) => changeIsBlank(event.target.checked)}></input>
            </label>
            <br />
            <label>
                Model Source:
                <SourceFileSelector onSelectedFile={(selectedFile) => changeSource(selectedFile)} />
            </label>
            <br />
            <button onClick={removeModel}>Remove Model</button>
        </li>
    );
};

export default Model;
