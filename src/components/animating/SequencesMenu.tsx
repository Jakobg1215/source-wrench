import { For, type Component } from 'solid-js';
import type { SetStoreFunction } from 'solid-js/store';
import { AnimationData } from './Animation';
import Sequence, { type SequenceData } from './Sequence';

type SequencesMenuProps = {
    sequences: SequenceData[];
    setSequences: SetStoreFunction<SequenceData[]>;
    animations: AnimationData[];
};

const SequencesMenu: Component<SequencesMenuProps> = (props) => {
    const addSequence = () => {
        props.setSequences([...props.sequences, newSequence()]);
    };

    let sequenceOrdinal = 0;
    const newSequence = (): SequenceData => {
        return {
            ordinal: sequenceOrdinal++,
            name: 'New Sequence',
            animation: '',
        };
    };

    return (
        <>
            <h3>Sequences</h3>
            <button onClick={addSequence}>Add Sequence</button>
            <ol>
                <For each={props.sequences}>
                    {(sequence) => <Sequence ordinal={sequence.ordinal} setSequences={props.setSequences} animations={props.animations} />}
                </For>
            </ol>
        </>
    );
};

export default SequencesMenu;
