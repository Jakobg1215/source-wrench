import type { Component } from 'solid-js';
import type { SetStoreFunction } from 'solid-js/store';
import SourceFileSelector from '../SourceFileSelector';

type AnimationProps = {
    readonly ordinal: number;
    setAnimations: SetStoreFunction<AnimationData[]>;
};

export type AnimationData = {
    readonly ordinal: number;
    name: string;
    source_file: string;
};

const Animation: Component<AnimationProps> = (props) => {
    const updateAnimationName = (value: string) => {
        props.setAnimations(
            (animation) => animation.ordinal === props.ordinal,
            'name',
            () => value,
        );
    };

    const updateAnimationFileSource = (value: string) => {
        props.setAnimations(
            (animation) => animation.ordinal === props.ordinal,
            'source_file',
            () => value,
        );
    };

    const removeAnimation = () => {
        props.setAnimations((animations) => animations.filter((animation) => animation.ordinal !== props.ordinal));
    };

    return (
        <li>
            <label>
                Animation Name:
                <input type="Text" value="New Animation" onChange={(event) => updateAnimationName(event.target.value)}></input>
            </label>
            <br />
            <label>
                Animation Source:
                <SourceFileSelector onSelectedFile={(selectedFile) => updateAnimationFileSource(selectedFile)} />
            </label>
            <br />
            <button onClick={removeAnimation}>Remove Animation</button>
        </li>
    );
};

export default Animation;
