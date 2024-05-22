import { For, type Component } from 'solid-js';
import type { SetStoreFunction } from 'solid-js/store';
import BodyPart, { type BodyPartData } from './BodyPart';

type BodyPartsMenuProps = {
    id: string;
    setBodyParts: SetStoreFunction<BodyPartData[]>;
    bodyParts: BodyPartData[];
};

const BodyPartsMenu: Component<BodyPartsMenuProps> = (props) => {
    const createNewBodyPart = () => {
        props.setBodyParts([...props.bodyParts, newBodyPart()]);
    };

    let bodyPartOrdinal = 0;
    const newBodyPart = (): BodyPartData => {
        return {
            ordinal: bodyPartOrdinal++,
            name: 'New Body Part',
            models: [],
        };
    };

    return (
        <section id={props.id}>
            <h2>Body Parts</h2>
            <button onClick={createNewBodyPart}>Add Part</button>
            <ol>
                <For each={props.bodyParts}>
                    {(bodyParts) => <BodyPart ordinal={bodyParts.ordinal} setBodyParts={props.setBodyParts} models={bodyParts.models} />}
                </For>
            </ol>
        </section>
    );
};

export default BodyPartsMenu;
