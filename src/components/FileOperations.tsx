import { invoke } from '@tauri-apps/api/core';
import { documentDir } from '@tauri-apps/api/path';
import { open } from '@tauri-apps/plugin-dialog';

type LoadedFile = LoadedFileData & {
    path: string;
};

type LoadedFileData = {
    skeleton: {
        name: string;
        parent: number | null;
    }[];
    animations: {
        name: string;
    }[];
    parts: {
        name: string;
    }[];
};

const loadedModelFiles: Map<string, number> = new Map();

const loadModelFile = async (previousPath: string): Promise<LoadedFile | null> => {
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
        return null;
    }

    manageLoadedModelFiles(previousPath, selectedFile.path);

    const loadedFiles: LoadedFileData | null = await invoke('load_file', { path: selectedFile.path });

    if (loadedFiles === null) {
        return null;
    }

    return {
        path: selectedFile.path,
        ...loadedFiles,
    };
};

const manageLoadedModelFiles = (previousPath: string, path: string) => {
    if (previousPath === path) {
        return;
    }

    const filePathCount = loadedModelFiles.get(path) ?? 0;
    loadedModelFiles.set(path, filePathCount + 1);

    const previousCount = loadedModelFiles.get(previousPath);

    if (previousCount === undefined) {
        return;
    }

    const currentCount = previousCount - 1;

    if (currentCount <= 0) {
        loadedModelFiles.delete(previousPath);
        invoke('unload_file', { path: previousPath });
        return;
    }

    loadedModelFiles.set(previousPath, currentCount);
};

export { loadModelFile };
