import type { Component } from 'solid-js';
import AnimationsMenu from './AnimationsMenu';
import SequencesMenu from './SequencesMenu';
import type { AnimationData } from './Animation';
import type { SetStoreFunction } from 'solid-js/store';
import type { SequenceData } from './Sequence';

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
