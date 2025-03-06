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

// These types should be synced with src-tauri\src\input.rs
type ImputedCompilationData = {
    model_name: string;
    export_path: string;
    body_parts: Record<string, ImputedBodyPart>;
    animations: Record<string, ImputedAnimation>;
    sequences: Record<string, ImputedSequence>;
};

type ImputedBodyPart = {
    models: Record<string, ImputedModel>;
};

type ImputedModel = {
    is_blank: boolean;
    file_source: string;
    part_names: Array<string>;
};

type ImputedAnimation = {
    file_source: string;
    animation_name: string;
};

type ImputedSequence = {
    animations: Array<Array<string>>;
};

const App: Component = () => {
    const [modelExportPath, setModelExportPath] = createSignal('');
    const [modelName, setModelName] = createSignal('');
    const [modelCompiling, setModelCompiling] = createSignal(false);
    const [bodyPartEntries, setBodyPartEntries] = createStore<BodyPartEntryProperties[]>([]);
    const [animationEntries, setAnimationEntries] = createStore<AnimationEntryProperties[]>([]);
    const [sequenceEntries, setSequenceEntries] = createStore<SequenceEntryProperties[]>([]);

    const compileModel = async () => {
        setModelCompiling(true);

        const data: ImputedCompilationData = {
            model_name: modelName(),
            export_path: modelExportPath(),
            body_parts: Object.fromEntries(
                bodyPartEntries.map((bodyPart) => [
                    bodyPart.data.name,
                    {
                        models: Object.fromEntries(
                            bodyPart.data.models.map((model) => [
                                model.data.name,
                                {
                                    is_blank: model.data.blank,
                                    file_source: model.data.file_source,
                                    part_names: model.data.part_names.filter((part) => part !== null),
                                },
                            ]),
                        ),
                    },
                ]),
            ),
            animations: Object.fromEntries(
                animationEntries.map((animation) => [
                    animation.data.name,
                    {
                        file_source: animation.data.file_source,
                        animation_name: animation.data.source_animation,
                    },
                ]),
            ),
            sequences: Object.fromEntries(
                sequenceEntries.map((sequence) => [
                    sequence.data.name,
                    {
                        animations: sequence.data.animations,
                    },
                ]),
            ),
        };

        await invoke('compile_model', { data });

        setModelCompiling(false);
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
                        <button disabled={modelCompiling()} onclick={async () => await compileModel()}>
                            Compile Model
                        </button>
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
