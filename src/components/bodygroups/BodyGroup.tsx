import { For, createEffect, type Component } from 'solid-js';
import { createStore, type SetStoreFunction } from 'solid-js/store';
import type { BodyPartData } from './BodyPart';
import BodyPart from './BodyPart';

type BodyGroupProps = {
    readonly ordinal: number;
    setBodyGroups: SetStoreFunction<BodyGroupData[]>;
    bodyParts: BodyPartData[];
};

export type BodyGroupData = {
    readonly ordinal: number;
    name: string;
    parts: BodyPartData[];
};

const BodyGroup: Component<BodyGroupProps> = (props) => {
    const [bodyParts, setbodyParts] = createStore<BodyPartData[]>([]);

    const createNewBodyPart = () => {
        setbodyParts((parts) => [...parts, newBodyPart()]);
    };

    let bodyPartOrdinal = 0;
    const newBodyPart = (): BodyPartData => {
        return {
            ordinal: bodyPartOrdinal++,
            name: 'New Body Part',
            is_blank: false,
            model_source: '',
        };
    };

    const updateBodyGroupBodyParts = () => {
        props.setBodyGroups(
            (bodyGroup) => bodyGroup.ordinal === props.ordinal,
            'parts',
            () => bodyParts,
        );
    };

    createEffect(() => {
        updateBodyGroupBodyParts();
    });

    const changeBodyGroupName = (newName: string) => {
        props.setBodyGroups(
            (bodyGroup) => bodyGroup.ordinal === props.ordinal,
            'name',
            () => newName,
        );
    };

    const removeBodyGroup = () => {
        props.setBodyGroups((bodyGroups) => bodyGroups.filter((bodyGroup) => bodyGroup.ordinal !== props.ordinal));
    };

    return (
        <li class="body-group">
            <label>
                Body Group Name:
                <input type="text" value="New Body Group" onChange={(event) => changeBodyGroupName(event.target.value)}></input>
            </label>
            <br />
            <button onClick={createNewBodyPart}>Add Body Part</button>
            <ol>
                <For each={props.bodyParts}>{(bodyPart) => <BodyPart ordinal={bodyPart.ordinal} setbodyParts={setbodyParts} />}</For>
            </ol>
            <button onClick={removeBodyGroup}>Remove Group</button>
        </li>
    );
};

export default BodyGroup;
