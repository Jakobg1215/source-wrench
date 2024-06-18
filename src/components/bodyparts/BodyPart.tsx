import { For, createEffect, type Component } from 'solid-js';
import { createStore, type SetStoreFunction } from 'solid-js/store';
import type { ModelData } from './Model';
import Model from './Model';

type BodyPartProps = {
    readonly ordinal: number;
    setBodyParts: SetStoreFunction<BodyPartData[]>;
    models: ModelData[];
};

export type BodyPartData = {
    readonly ordinal: number;
    name: string;
    models: ModelData[];
};

const BodyPart: Component<BodyPartProps> = (props) => {
    const [bodyParts, setModels] = createStore<ModelData[]>([]);

    const createNewModel = () => {
        setModels((parts) => [...parts, newModel()]);
    };

    let modelOrdinal = 0;
    const newModel = (): ModelData => {
        return {
            ordinal: modelOrdinal++,
            name: 'New Model',
            is_blank: false,
            model_source: '',
            part_name: [],
        };
    };

    const updateBodyPartModels = () => {
        props.setBodyParts(
            (bodyPart) => bodyPart.ordinal === props.ordinal,
            'models',
            () => bodyParts,
        );
    };

    createEffect(() => {
        updateBodyPartModels();
    });

    const changeBodyPartName = (newName: string) => {
        props.setBodyParts(
            (bodyPart) => bodyPart.ordinal === props.ordinal,
            'name',
            () => newName,
        );
    };

    const removeBodyPart = () => {
        props.setBodyParts((bodyParts) => bodyParts.filter((bodyPart) => bodyPart.ordinal !== props.ordinal));
    };

    return (
        <li class="body-part">
            <label>
                Body Part Name:
                <input type="text" value="New Body Part" onChange={(event) => changeBodyPartName(event.target.value)}></input>
            </label>
            <br />
            <button onClick={createNewModel}>Add Model</button>
            <ol>
                <For each={props.models}>{(model) => <Model ordinal={model.ordinal} setModels={setModels} />}</For>
            </ol>
            <button onClick={removeBodyPart}>Remove Part</button>
        </li>
    );
};

export default BodyPart;
