import { invoke } from '@tauri-apps/api/core';
import { documentDir } from '@tauri-apps/api/path';
import { open } from '@tauri-apps/plugin-dialog';
import { createSignal, Show, type Component } from 'solid-js';
import { createStore } from 'solid-js/store';
import { AnimationEntryProperties } from './components/AnimationEntry';
import AnimationMenu from './components/AnimationMenu';
import { BodyPartEntryProperties } from './components/BodyPartEntry';
import BodyPartMenu from './components/BodyPartMenu';
import Logging from './components/Logging';
import { SequenceEntryProperties } from './components/SequenceEntry';
import SequenceMenu from './components/SequenceMenu';

type ImputedCompilationData = {
    model_name: string;
    export_path: string;
    body_parts: {
        name: string;
        models: {
            name: string;
            is_blank: boolean;
            file_source: string;
            part_names: (string | null)[];
        }[];
    }[];
    animations: {
        name: string;
        file_source: string;
        animation_name: string;
    }[];
    sequences: {
        name: string;
        animations: string[];
    }[];
};

const App: Component = () => {
    const [modelExportPath, setModelExportPath] = createSignal('');
    const [modelName, setModelName] = createSignal('');
    const [bodyPartEntries, setBodyPartEntries] = createStore<BodyPartEntryProperties[]>([]);
    const [animationEntries, setAnimationEntries] = createStore<AnimationEntryProperties[]>([]);
    const [sequenceEntries, setSequenceEntries] = createStore<SequenceEntryProperties[]>([]);

    const compileModel = async () => {
        const data: ImputedCompilationData = {
            model_name: modelName(),
            export_path: modelExportPath(),
            body_parts: bodyPartEntries.map((bodyPart) => ({
                name: bodyPart.data.name,
                models: bodyPart.data.models.map((model) => ({
                    name: model.data.name,
                    is_blank: model.data.blank,
                    file_source: model.data.file_source,
                    part_names: model.data.part_names,
                })),
            })),
            animations: animationEntries.map((animation) => ({
                name: animation.data.name,
                file_source: animation.data.file_source,
                animation_name: animation.data.source_animation,
            })),
            sequences: sequenceEntries.map((sequence) => ({
                name: sequence.data.name,
                animations: sequence.data.animations.flat(1),
            })),
        };

        invoke('compile_model', { data });
    };

    return (
        <>
            <header>
                <nav>
                    <ul>
                        <li>
                            <a href="#Compile-Menu">Compilation</a>
                        </li>
                        <li>
                            <a href="#Body-Part-Menu">Body Parts</a>
                        </li>
                        <li>
                            <a href="#Animation-Menu">Animations</a>
                        </li>
                        <li>
                            <a href="Sequence-Menu">Sequences</a>
                        </li>
                    </ul>
                </nav>
            </header>
            <main>
                <h1>Source Wrench</h1>
                <section id="Compile-Menu">
                    <label>
                        Export Path
                        <input
                            name="ExportPath"
                            type="text"
                            readonly
                            value={modelExportPath()}
                            onClick={async () => {
                                const selectedFile = await open({
                                    defaultPath: await documentDir(),
                                    directory: true,
                                    title: 'Model Export Path',
                                });

                                if (selectedFile === null) {
                                    setModelExportPath('');
                                    return;
                                }

                                setModelExportPath(selectedFile);
                            }}
                        />
                    </label>
                    <Show when={modelExportPath()}>
                        <br />
                        <label>
                            Model Name
                            <input name="ModelName" type="text" onChange={(event) => setModelName(event.target.value)} />
                        </label>
                        <br />
                        <button onclick={() => compileModel()}>Compile Model</button>
                    </Show>
                </section>
                <Logging />
                <BodyPartMenu bodyPartEntries={bodyPartEntries} setBodyPartEntries={setBodyPartEntries} />
                <AnimationMenu animationEntries={animationEntries} setAnimationEntries={setAnimationEntries} />
                <SequenceMenu sequenceEntries={sequenceEntries} setSequenceEntries={setSequenceEntries} />
            </main>
        </>
    );
};

export default App;
