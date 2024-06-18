import { invoke } from '@tauri-apps/api/core';
import { documentDir } from '@tauri-apps/api/path';
import { open } from '@tauri-apps/plugin-dialog';
import { createSignal, type Component } from 'solid-js';

type SourceFileSelectorProps = {
    onSelectedFile: (path: string) => void;
};

const SourceFileSelector: Component<SourceFileSelectorProps> = (props) => {
    const [selectedFile, setSelectedFile] = createSignal('');

    return (
        <>
            <input
                type="text"
                value={selectedFile()}
                readonly
                onClick={async () => {
                    const selectedFile = await open({
                        defaultPath: await documentDir(),
                        directory: false,
                        filters: [
                            {
                                extensions: ['smd'],
                                name: 'Supported Files',
                            },
                        ],
                        multiple: false,
                        title: 'Select Source File',
                    });

                    if (selectedFile === null) {
                        return;
                    }

                    if (!(await invoke('load_file', { path: selectedFile.path }))) {
                        return;
                    }

                    props.onSelectedFile(selectedFile.path);

                    setSelectedFile(() => selectedFile.path);
                }}
            ></input>
        </>
    );
};

export default SourceFileSelector;
