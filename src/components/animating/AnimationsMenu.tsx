import { For, type Component } from 'solid-js';
import type { SetStoreFunction } from 'solid-js/store';
import Animation, { type AnimationData } from './Animation';

type AnimationsMenuProps = {
    animations: AnimationData[];
    setAnimations: SetStoreFunction<AnimationData[]>;
};

const AnimationsMenu: Component<AnimationsMenuProps> = (props) => {
    const addAnimation = () => {
        props.setAnimations([...props.animations, newAnimation()]);
    };

    let animationOrdinal = 0;
    const newAnimation = (): AnimationData => {
        return {
            ordinal: animationOrdinal++,
            name: 'New Animation',
            source_file: '',
            animation_name: '',
        };
    };

    return (
        <>
            <h3>Animations</h3>
            <button onClick={addAnimation}>Add Animation</button>
            <button>Add Animation Group</button>
            <ol>
                <For each={props.animations}>{(animation) => <Animation ordinal={animation.ordinal} setAnimations={props.setAnimations} />}</For>
            </ol>
        </>
    );
};

export default AnimationsMenu;
