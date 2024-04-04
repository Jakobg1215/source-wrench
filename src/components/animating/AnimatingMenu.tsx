import type { Component } from 'solid-js';
import type { SetStoreFunction } from 'solid-js/store';
import type { AnimationData } from './Animation';
import AnimationsMenu from './AnimationsMenu';
import type { SequenceData } from './Sequence';
import SequencesMenu from './SequencesMenu';

type AnimatingMenuProps = {
    id: string;
    animations: AnimationData[];
    setAnimations: SetStoreFunction<AnimationData[]>;
    sequences: SequenceData[];
    setSequences: SetStoreFunction<SequenceData[]>;
};

const AnimatingMenu: Component<AnimatingMenuProps> = (props) => {
    return (
        <section id={props.id}>
            <h2>Animating</h2>
            <AnimationsMenu animations={props.animations} setAnimations={props.setAnimations} />
            <SequencesMenu sequences={props.sequences} setSequences={props.setSequences} animations={props.animations} />
        </section>
    );
};

export default AnimatingMenu;
