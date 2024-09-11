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
                <nav class="bg-white border-gray-200 dark:bg-gray-900">
                    <div class="max-w-screen-xl flex flex-wrap items-center justify-between mx-auto p-4">
                        <div class="flex items-center space-x-3 rtl:space-x-reverse">
                            <span class="self-center text-2xl font-semibold whitespace-nowrap dark:text-white">SourceWrench</span>
                        </div>
                        <div class="flex md:order-2 space-x-3 gap-3 md:space-x-0 rtl:space-x-reverse">
                            <p class="self-center dark:text-white mr-2">{modelExportPath()}</p>
                            <button
                                type="button"
                                class="text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-4 py-2 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800"
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
                            >
                                Export Path
                            </button>

                            <button
                                type="button"
                                class="text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-4 py-2 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800"
                                onclick={() => compileModel()}
                            >
                                Compile Model
                            </button>
                        </div>
                    </div>
                </nav>
            </header>
            <main>
                <section id="Compile-Menu">
                    <Show when={modelExportPath()}>
                        <br />
                        <div>
                            <label for="first_name" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">
                                Model Name
                            </label>
                            <input
                                type="text"
                                id="first_name"
                                onChange={(event) => setModelName(event.target.value)}
                                class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
                                placeholder="idkman"
                                required
                            />
                        </div>
                    </Show>
                </section>
                <BodyPartMenu bodyPartEntries={bodyPartEntries} setBodyPartEntries={setBodyPartEntries} />
                <AnimationMenu animationEntries={animationEntries} setAnimationEntries={setAnimationEntries} />
                <SequenceMenu sequenceEntries={sequenceEntries} setSequenceEntries={setSequenceEntries} />

                <div class="w-screen">
                    <Logging />
                </div>
            </main>
        </>
    );
};

export default App;
