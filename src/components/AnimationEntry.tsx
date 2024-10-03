import { createSignal, For, Show, type Component } from 'solid-js';
import { type SetStoreFunction } from 'solid-js/store';
import { loadModelFile, unloadModelFile } from './FileOperations';

type AnimationEntryProperties = {
    readonly identifier: number;
    readonly setAnimationEntries: SetStoreFunction<AnimationEntryProperties[]>;
    readonly data: {
        name: string;
        file_source: string;
        source_animation: string;
    };
};

export type { AnimationEntryProperties };

const AnimationEntry: Component<AnimationEntryProperties> = (properties) => {
    const [selectedFile, setSelectedFile] = createSignal('');
    const [availableAnimations, setAvailableAnimations] = createSignal<string[]>([]);

    const removeAnimation = () => {
        unloadModelFile(selectedFile());
        properties.setAnimationEntries((animations) => animations.filter((animation) => animation.identifier !== properties.identifier));
    };

    const changeAnimationName = (name: string) => {
        properties.setAnimationEntries((animation) => animation.identifier == properties.identifier, 'data', 'name', name);
    };

    const changeAnimationFileSource = (fileSource: string) => {
        properties.setAnimationEntries((animation) => animation.identifier == properties.identifier, 'data', 'file_source', fileSource);
    };

    const changeAnimationSourceAnimation = (sourceAnimation: string) => {
        properties.setAnimationEntries((animation) => animation.identifier == properties.identifier, 'data', 'source_animation', sourceAnimation);
    };

    return (
        <div class="Animation-Entry">
            <h3>Animation</h3>
            <label>
                Name:
                <input name="AnimationName" type="text" value={properties.data.name} onChange={(event) => changeAnimationName(event.target.value)} />
            </label>
            <br />
            <label>
                File:
                <input
                    name="AnimationFileSource"
                    type="text"
                    value={selectedFile()}
                    readonly
                    onClick={async () => {
                        const loadedFile = await loadModelFile(selectedFile());

                        if (loadedFile === null) {
                            return;
                        }

                        changeAnimationFileSource(loadedFile.path);

                        const animations = loadedFile.animations.map((animation) => animation.name);
                        changeAnimationSourceAnimation(animations[0]!);
                        setAvailableAnimations(animations);
                        setSelectedFile(() => loadedFile.path);
                    }}
                />
            </label>
            <br />
            <Show when={availableAnimations().length > 0}>
                <label>
                    Animation:
                    <select name="AnimationSourceAnimation" onChange={(event) => changeAnimationFileSource(event.target.value)}>
                        <For each={availableAnimations()}>{(animation) => <option value={animation}>{animation}</option>}</For>
                    </select>
                </label>
            </Show>
            <br />
            <button onClick={() => removeAnimation()}>Remove</button>
        </div>
    );
};

export default AnimationEntry;
