import { invoke } from '@tauri-apps/api/core';
import { documentDir } from '@tauri-apps/api/path';
import { open } from '@tauri-apps/plugin-dialog';
import { createSignal, type Component } from 'solid-js';
import { createStore } from 'solid-js/store';
import Logging from './components/Logging';
import AnimatingMenu from './components/animating/AnimatingMenu';
import { AnimationData } from './components/animating/Animation';
import { SequenceData } from './components/animating/Sequence';
import { BodyPartData } from './components/bodyparts/BodyPart';
import BodyPartsMenu from './components/bodyparts/BodyPartsMenu';

type CompilationData = {
    model_name: string;
    body_parts: Array<BodyPartData>;
    animations: Array<AnimationData>;
    sequences: Array<SequenceData>;
    export_path: string;
};

const App: Component = () => {
    const [modelName, setModelName] = createSignal('');
    const [bodyParts, setBodyParts] = createStore<BodyPartData[]>([]);
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
            body_parts: bodyParts,
            animations,
            sequences,
            export_path: selectedDirectory,
        };

        invoke('compile_model', { data });
    };

    return (
        <>
            <div class="MainMenu">
                <header>
                    <h1>Source Wrench</h1>
                    <nav>
                        <ul>
                            <li>
                                <a href="#bodyparts">Body Parts</a>
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
                    <BodyPartsMenu id="bodyparts" setBodyParts={setBodyParts} bodyParts={bodyParts} />
                    <AnimatingMenu id="animating" animations={animations} setAnimations={setAnimations} sequences={sequences} setSequences={setSequences} />
                </main>
            </div>

            <Logging />
        </>
    );
};

export default App;
