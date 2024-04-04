import type { Component } from 'solid-js';
import type { SetStoreFunction } from 'solid-js/store';

type BodyPartProps = {
    readonly ordinal: number;
    setbodyParts: SetStoreFunction<BodyPartData[]>;
};

export type BodyPartData = {
    readonly ordinal: number;
    name: string;
    is_blank: boolean;
    model_source: string;
};

const BodyPart: Component<BodyPartProps> = (props) => {
    const changeBodyPartName = (newName: string) => {
        props.setbodyParts(
            (bodyPart) => bodyPart.ordinal === props.ordinal,
            'name',
            () => newName
        );
    };

    const changeBodyPartIsBlank = (value: boolean) => {
        props.setbodyParts(
            (bodyPart) => bodyPart.ordinal === props.ordinal,
            'is_blank',
            () => value
        );
    };

    const changeBodyPartModelSource = (value: string) => {
        props.setbodyParts(
            (bodyPart) => bodyPart.ordinal === props.ordinal,
            'model_source',
            () => value
        );
    };

    const removeBodyPart = () => {
        props.setbodyParts((bodyParts) => bodyParts.filter((bodyPart) => bodyPart.ordinal !== props.ordinal));
    };

    return (
        <li class="body-part">
            <label>
                Part Name:
                <input type="text" value="New Body Part" onChange={(event) => changeBodyPartName(event.target.value)}></input>
            </label>
            <br />
            <label>
                Is Blank
                <input type="checkbox" onChange={(event) => changeBodyPartIsBlank(event.target.checked)}></input>
            </label>
            <br />
            <label>
                Part Model Source:
                <input type="text" onChange={(event) => changeBodyPartModelSource(event.target.value)}></input>
            </label>
            <br />
            <button onClick={removeBodyPart}>Remove Part</button>
        </li>
    );
};

export default BodyPart;
