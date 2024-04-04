import { invoke } from '@tauri-apps/api/core';
import { createSignal, type Component } from 'solid-js';
import BodyGroupsMenu from './components/bodygroups/BodyGroupsMenu';
import { BodyGroupData } from './components/bodygroups/BodyGroup';
import { createStore } from 'solid-js/store';

type CompilationData = {
    model_name: string;
    body_groups: Array<BodyGroupData>;
};

const App: Component = () => {
    const [modelName, setModelName] = createSignal('');
    const [bodyGroups, setBodyGroups] = createStore<BodyGroupData[]>([]);

    const compileModel = () => {
        const data: CompilationData = {
            model_name: modelName(),
            body_groups: bodyGroups,
        };

        invoke('compile_model', { data });
    };

    return (
        <>
            <header>
                <h1>Source Wrench</h1>
                <nav>
                    <ul>
                        <li>
                            <a href="#bodygroups">Body Groups</a>
                        </li>
                    </ul>
                </nav>
            </header>

            <main>
                <button onClick={compileModel}>Compile Model</button>
                <br />
                <label>
                    Model Name:
                    <input type="text" onChange={(event) => setModelName(event.target.value)}></input>
                    .mdl
                </label>
                <BodyGroupsMenu id="bodygroups" setBodyGroups={setBodyGroups} bodyGroups={bodyGroups} />
            </main>
        </>
    );
};

export default App;
