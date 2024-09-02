import { For, type Component } from 'solid-js';
import { type SetStoreFunction } from 'solid-js/store';
import SequenceEntry, { type SequenceEntryProperties } from './SequenceEntry';

type SequenceMenuProperties = {
    sequenceEntries: SequenceEntryProperties[];
    setSequenceEntries: SetStoreFunction<SequenceEntryProperties[]>;
};

export type { SequenceMenuProperties };

const SequenceMenu: Component<SequenceMenuProperties> = (properties) => {
    const addSequence = () => {
        properties.setSequenceEntries([...properties.sequenceEntries, createNewSequence()]);
    };

    let animationEntrySequenceGenerator = 0;
    const createNewSequence = (): SequenceEntryProperties => {
        return {
            identifier: animationEntrySequenceGenerator++,
            setSequenceEntries: properties.setSequenceEntries,
            data: {
                name: 'New Sequence',
                animations: [],
            },
        };
    };

    return (
        <section id="Sequence-Menu">
            <h2>Sequences</h2>
            <button onClick={() => addSequence()}>Add Sequence</button>
            <For each={properties.sequenceEntries}>
                {({ identifier, setSequenceEntries, data }) => <SequenceEntry identifier={identifier} setSequenceEntries={setSequenceEntries} data={data} />}
            </For>
        </section>
    );
};

export default SequenceMenu;
