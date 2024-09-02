import { type Component, For } from 'solid-js';
import { type SetStoreFunction } from 'solid-js/store';
import BodyPartEntry, { type BodyPartEntryProperties } from './BodyPartEntry';

type BodyPartMenuProperties = {
    bodyPartEntries: BodyPartEntryProperties[];
    setBodyPartEntries: SetStoreFunction<BodyPartEntryProperties[]>;
};

export type { BodyPartMenuProperties };

const BodyPartMenu: Component<BodyPartMenuProperties> = (properties) => {
    const addBodyPart = () => {
        properties.setBodyPartEntries([...properties.bodyPartEntries, createNewBodyPart()]);
    };

    let bodyPartIdentifierGenerator = 0;
    const createNewBodyPart = (): BodyPartEntryProperties => {
        return {
            identifier: bodyPartIdentifierGenerator++,
            setBodyParts: properties.setBodyPartEntries,
            data: {
                name: 'New Body Part',
                models: [],
            },
        };
    };

    return (
        <section id="Body-Part-Menu">
            <h2>Body Parts</h2>
            <button onClick={() => addBodyPart()}>Add Part</button>
            <For each={properties.bodyPartEntries}>
                {({ identifier, setBodyParts, data }) => <BodyPartEntry identifier={identifier} setBodyParts={setBodyParts} data={data} />}
            </For>
        </section>
    );
};

export default BodyPartMenu;
