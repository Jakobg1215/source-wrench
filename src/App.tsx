import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { documentDir } from '@tauri-apps/api/path';
import { open } from '@tauri-apps/plugin-dialog';
import { createSignal, type Component } from 'solid-js';
import { createStore } from 'solid-js/store';
import Logging from './components/Logging';
import AnimatingMenu from './components/animating/AnimatingMenu';
import { AnimationData } from './components/animating/Animation';
import { SequenceData } from './components/animating/Sequence';
import { BodyGroupData } from './components/bodygroups/BodyGroup';
import BodyGroupsMenu from './components/bodygroups/BodyGroupsMenu';

type CompilationData = {
    model_name: string;
    body_groups: Array<BodyGroupData>;
    animations: Array<AnimationData>;
    sequences: Array<SequenceData>;
    export_path: string;
};

const App: Component = () => {
    const [modelName, setModelName] = createSignal('');
    const [bodyGroups, setBodyGroups] = createStore<BodyGroupData[]>([]);
    const [animations, setAnimations] = createStore<AnimationData[]>([]);
    const [sequences, setSequences] = createStore<SequenceData[]>([]);

    const compileModel = async () => {
        const selectedDirectory = await open({
            defaultPath: await documentDir(),
            directory: true,
            title: 'Select Output Directory',
        });

        if (selectedDirectory === null) {
            return;
        }

        const data: CompilationData = {
            model_name: modelName(),
            body_groups: bodyGroups,
            animations,
            sequences,
            export_path: selectedDirectory,
        };

        invoke('compile_model', { data });
    };

    listen('source-wrench-log', (value) => {
        console.log(value);
    });

    return (
        <>
            <div class="MainMenu">
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
            </div>

            <Logging />
        </>
    );
};

export default App;
