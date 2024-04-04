import { For, type Component } from 'solid-js';
import type { SetStoreFunction } from 'solid-js/store';
import { AnimationData } from './Animation';

type SequenceProps = {
    readonly ordinal: number;
    setSequences: SetStoreFunction<SequenceData[]>;
    animations: AnimationData[];
};

export type SequenceData = {
    readonly ordinal: number;
    name: string;
    animation: string;
};

const Sequence: Component<SequenceProps> = (props) => {
    const updateSequenceName = (value: string) => {
        props.setSequences(
            (sequence) => sequence.ordinal === props.ordinal,
            'name',
            () => value,
        );
    };

    const updateSequenceAnimation = (value: string) => {
        props.setSequences(
            (sequence) => sequence.ordinal === props.ordinal,
            'animation',
            () => value,
        );
    };

    const removeSequence = () => {
        props.setSequences((sequences) => sequences.filter((sequence) => sequence.ordinal !== props.ordinal));
    };

    return (
        <li>
            <label>
                Sequence Name:
                <input type="Text" value="New Sequence" onChange={(event) => updateSequenceName(event.target.value)}></input>
            </label>
            <br />
            <label>
                Animation:
                <input list="animations" onChange={(event) => updateSequenceAnimation(event.target.value)}></input>
                <datalist id="animations">
                    <For each={props.animations}>{(animation) => <option value={animation.name} />}</For>
                </datalist>
            </label>
            <br />
            <button onClick={removeSequence}>Remove Sequence</button>
        </li>
    );
};

export default Sequence;
