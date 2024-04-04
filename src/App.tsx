import { invoke } from '@tauri-apps/api/core';
import { createSignal, type Component } from 'solid-js';
import BodyGroupsMenu from './components/bodygroups/BodyGroupsMenu';
import { BodyGroupData } from './components/bodygroups/BodyGroup';
import { createStore } from 'solid-js/store';
import AnimatingMenu from './components/animating/AnimatingMenu';
import { AnimationData } from './components/animating/Animation';
import { SequenceData } from './components/animating/Sequence';

type CompilationData = {
    model_name: string;
    body_groups: Array<BodyGroupData>;
    animations: Array<AnimationData>;
    sequences: Array<SequenceData>;
};

const App: Component = () => {
    const [modelName, setModelName] = createSignal('');
    const [bodyGroups, setBodyGroups] = createStore<BodyGroupData[]>([]);
    const [animations, setAnimations] = createStore<AnimationData[]>([]);
    const [sequences, setSequences] = createStore<SequenceData[]>([]);

    const compileModel = () => {
        const data: CompilationData = {
            model_name: modelName(),
            body_groups: bodyGroups,
            animations,
            sequences,
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
                        <li>
                            <a href="#animating">Animating</a>
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
                <AnimatingMenu id="animating" animations={animations} setAnimations={setAnimations} sequences={sequences} setSequences={setSequences} />
            </main>
        </>
    );
};

export default App;
