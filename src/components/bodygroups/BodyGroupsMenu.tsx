import { For, type Component } from 'solid-js';
import { createStore } from 'solid-js/store';
import BodyGroup, { type BodyGroupData } from './BodyGroup';

type BodyGroupsMenuProps = {
    id: string;
};

const BodyGroupsMenu: Component<BodyGroupsMenuProps> = (props) => {
    const [bodyGroups, setBodyGroups] = createStore<BodyGroupData[]>([]);

    const createNewBodyGroup = () => {
        setBodyGroups([...bodyGroups, newBodyGroup()]);
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
                <For each={bodyGroups}>
                    {(bodyGroup) => <BodyGroup ordinal={bodyGroup.ordinal} setBodyGroups={setBodyGroups} bodyParts={bodyGroup.parts} />}
                </For>
            </ol>
        </section>
    );
};

export default BodyGroupsMenu;
