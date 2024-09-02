import { type Component, createSignal, For, Show } from 'solid-js';
import { type SetStoreFunction } from 'solid-js/store';
import { loadModelFile } from './FileOperations';

type BodyPartModelEntryProperties = {
    readonly identifier: number;
    readonly setBodyPartModels: SetStoreFunction<BodyPartModelEntryProperties[]>;
    readonly data: {
        name: string;
        blank: boolean;
        file_source: string;
        part_names: (string | null)[];
    };
};

export type { BodyPartModelEntryProperties };

const BodyPartModelEntry: Component<BodyPartModelEntryProperties> = (properties) => {
    const [isBlank, setIsBlank] = createSignal(properties.data.blank);
    const [availableParts, setAvailableParts] = createSignal<string[]>([]);
    const [selectedFile, setSelectedFile] = createSignal('');

    const removeBodyPartModel = () => {
        properties.setBodyPartModels((bodyPartModel) => bodyPartModel.filter((model) => model.identifier !== properties.identifier));
    };

    const changeBodyPartModelName = (name: string) => {
        properties.setBodyPartModels((bodyPartModel) => bodyPartModel.identifier === properties.identifier, 'data', 'name', name);
    };

    const changeBodyPartModelBlank = (blank: boolean) => {
        properties.setBodyPartModels((bodyPartModel) => bodyPartModel.identifier === properties.identifier, 'data', 'blank', blank);
        setIsBlank(blank);
    };

    const changeBodyPartModelFileSource = (fileSource: string) => {
        properties.setBodyPartModels((bodyPartModel) => bodyPartModel.identifier === properties.identifier, 'data', 'file_source', fileSource);
    };

    const setBodyPartModelParts = (parts: string[]) => {
        properties.setBodyPartModels((bodyPartModel) => bodyPartModel.identifier === properties.identifier, 'data', 'part_names', parts);
    };

    const changeBodyPartModelPart = (partIndex: number, value: string | null) => {
        properties.setBodyPartModels((bodyPartModel) => bodyPartModel.identifier === properties.identifier, 'data', 'part_names', partIndex, value);
    };

    return (
        <div class="Body-Part-Model-Entry">
            <h5>Model</h5>
            <label>
                Blank
                <input name="BodyPartModelBlank" type="checkbox" onChange={() => changeBodyPartModelBlank(!isBlank())} checked={isBlank()}></input>
            </label>
            <Show when={!isBlank()}>
                <br />
                <label>
                    Name:
                    <input
                        name="BodyPartModelName"
                        type="text"
                        onChange={(event) => changeBodyPartModelName(event.target.value)}
                        value={properties.data.name}
                    />
                </label>
                <br />
                <label>
                    File:
                    <input
                        name="BodyPartModelFileSource"
                        type="text"
                        value={selectedFile()}
                        readonly
                        onClick={async () => {
                            const loadedFile = await loadModelFile(selectedFile());

                            if (loadedFile === null) {
                                return;
                            }

                            changeBodyPartModelFileSource(loadedFile.path);

                            const parts = loadedFile.parts.map((part) => part.name);
                            setBodyPartModelParts(parts);
                            setAvailableParts(parts);
                            setSelectedFile(() => loadedFile.path);
                        }}
                    />
                </label>
                <br />
                <For each={availableParts()}>
                    {(partName, index) => (
                        <div>
                            <label>
                                {partName}:
                                <input
                                    name={`BodyPartModelPart${index()}`}
                                    type="checkbox"
                                    checked={properties.data.part_names[index()] === null ? false : true}
                                    onChange={(event) => {
                                        changeBodyPartModelPart(index(), event.target.checked ? partName : null);
                                    }}
                                ></input>
                            </label>
                        </div>
                    )}
                </For>
            </Show>
            <br />
            <button onClick={() => removeBodyPartModel()}>Remove</button>
        </div>
    );
};

export default BodyPartModelEntry;
