import { createSignal, type Component } from 'solid-js';
import { open } from '@tauri-apps/plugin-dialog';
import { documentDir } from '@tauri-apps/api/path';

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

                    props.onSelectedFile(selectedFile.path);

                    setSelectedFile(() => selectedFile.path);
                }}
            ></input>
        </>
    );
};

export default SourceFileSelector;
