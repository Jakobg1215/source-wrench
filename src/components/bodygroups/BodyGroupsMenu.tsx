import { For, type Component } from 'solid-js';
import type { SetStoreFunction } from 'solid-js/store';
import BodyGroup, { type BodyGroupData } from './BodyGroup';

type BodyGroupsMenuProps = {
    id: string;
    setBodyGroups: SetStoreFunction<BodyGroupData[]>;
    bodyGroups: BodyGroupData[];
};

const BodyGroupsMenu: Component<BodyGroupsMenuProps> = (props) => {
    const createNewBodyGroup = () => {
        props.setBodyGroups([...props.bodyGroups, newBodyGroup()]);
    };

    let bodyGroupOrdinal = 0;
    const newBodyGroup = (): BodyGroupData => {
        return {
            ordinal: bodyGroupOrdinal++,
            name: 'New Body Group',
            parts: [],
        };
    };

    return (
        <section id={props.id}>
            <h2>Body Groups</h2>
            <button onClick={createNewBodyGroup}>Add Group</button>
            <ol>
                <For each={props.bodyGroups}>
                    {(bodyGroup) => <BodyGroup ordinal={bodyGroup.ordinal} setBodyGroups={props.setBodyGroups} bodyParts={bodyGroup.parts} />}
                </For>
            </ol>
        </section>
    );
};

export default BodyGroupsMenu;
