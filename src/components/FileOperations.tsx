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
                extensions: ['smd', 'obj'],
                name: 'Supported Files',
            },
        ],
        multiple: false,
        title: 'Select Source File',
    });

    if (selectedFile === null) {
        return null;
    }

    const loadedFiles: LoadedFileData | null = await invoke('load_file', { path: selectedFile });

    if (loadedFiles === null) {
        return null;
    }

    await manageLoadedModelFiles(previousPath, selectedFile);

    return {
        path: selectedFile,
        ...loadedFiles,
    };
};

const manageLoadedModelFiles = async (previousPath: string, path: string) => {
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
        await invoke('unload_file', { path: previousPath });
        return;
    }

    loadedModelFiles.set(previousPath, currentCount);
};

const unloadModelFile = async (path: string) => {
    if (path === '') {
        return;
    }

    const filePathCount = loadedModelFiles.get(path) ?? 1;
    const newCount = filePathCount - 1;
    if (newCount === 0) {
        loadedModelFiles.delete(path);
        await invoke('unload_file', { path });
        return;
    }

    loadedModelFiles.set(path, newCount);
};

addEventListener('beforeunload', async () => {
    for (const [path] of loadedModelFiles) {
        await invoke('unload_file', { path });
    }

    loadedModelFiles.clear();
});

export { loadModelFile, unloadModelFile };
