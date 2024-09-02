import { For, type Component } from 'solid-js';
import { createStore, type SetStoreFunction } from 'solid-js/store';
import BodyPartModelEntry, { type BodyPartModelEntryProperties } from './BodyPartModelEntry';

type BodyPartEntryProperties = {
    readonly identifier: number;
    readonly setBodyParts: SetStoreFunction<BodyPartEntryProperties[]>;
    readonly data: {
        name: string;
        models: BodyPartModelEntryProperties[];
    };
};

export type { BodyPartEntryProperties };

const BodyPartEntry: Component<BodyPartEntryProperties> = (properties) => {
    const [bodyPartModelEntries, setBodyPartModelEntries] = createStore<BodyPartModelEntryProperties[]>([]);

    const addBodyPartModel = () => {
        setBodyPartModelEntries([...bodyPartModelEntries, createNewBodyPartModel()]);
        properties.setBodyParts((bodyPart) => bodyPart.identifier === properties.identifier, 'data', 'models', bodyPartModelEntries);
    };

    let bodyPartModelIdentifierGenerator = 0;
    const createNewBodyPartModel = (): BodyPartModelEntryProperties => {
        return {
            identifier: bodyPartModelIdentifierGenerator++,
            setBodyPartModels: setBodyPartModelEntries,
            data: {
                name: 'New Model',
                blank: false,
                file_source: '',
                part_names: [],
            },
        };
    };

    const removeBodyPart = () => {
        properties.setBodyParts((bodyParts) => bodyParts.filter((bodyPart) => bodyPart.identifier !== properties.identifier));
    };

    const changeBodyPartName = (name: string) => {
        properties.setBodyParts((bodyPart) => bodyPart.identifier === properties.identifier, 'data', 'name', name);
    };

    return (
        <div class="Body-Part-Entry">
            <h3>Body Part</h3>
            <label>
                Name:
                <input name="BodyPartName" type="text" value={properties.data.name} onChange={(event) => changeBodyPartName(event.target.value)} />
            </label>
            <br />
            <button onClick={() => addBodyPartModel()}>Add Model</button>
            <button onClick={() => removeBodyPart()}>Remove</button>
            <h4>Models</h4>
            <For each={bodyPartModelEntries}>
                {({ identifier, setBodyPartModels, data }) => <BodyPartModelEntry identifier={identifier} setBodyPartModels={setBodyPartModels} data={data} />}
            </For>
        </div>
    );
};

export default BodyPartEntry;
