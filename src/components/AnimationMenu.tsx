import { type Component, For } from 'solid-js';
import { type SetStoreFunction } from 'solid-js/store';
import AnimationEntry, { type AnimationEntryProperties } from './AnimationEntry';

type AnimationMenuProperties = {
    animationEntries: AnimationEntryProperties[];
    setAnimationEntries: SetStoreFunction<AnimationEntryProperties[]>;
};

export type { AnimationMenuProperties };

const AnimationMenu: Component<AnimationMenuProperties> = (properties) => {
    const addAnimation = () => {
        properties.setAnimationEntries([...properties.animationEntries, createNewAnimation()]);
    };

    let animationEntryIdentifierGenerator = 0;
    const createNewAnimation = (): AnimationEntryProperties => {
        return {
            identifier: animationEntryIdentifierGenerator++,
            setAnimationEntries: properties.setAnimationEntries,
            data: {
                name: 'New Animation',
                file_source: '',
                source_animation: '',
            },
        };
    };

    return (
        <section id="Animation-Menu">
            <h2>Animations</h2>
            <button onClick={() => addAnimation()}>Add Animation</button>
            <For each={properties.animationEntries}>
                {({ identifier, setAnimationEntries, data }) => (
                    <AnimationEntry identifier={identifier} setAnimationEntries={setAnimationEntries} data={data} />
                )}
            </For>
            <datalist id="Animation-Names">
                <For each={properties.animationEntries}>{({ data }) => <option value={data.name}></option>}</For>
            </datalist>
        </section>
    );
};

export default AnimationMenu;
