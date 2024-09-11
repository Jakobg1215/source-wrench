import { listen } from '@tauri-apps/api/event';
import { For, createSignal, type Component } from 'solid-js';

type LogEvent = {
    level: 'Log' | 'Info' | 'Verbose' | 'Debug' | 'Warn' | 'Error';
    message: string;
};

const SourceFileSelector: Component = () => {
    const [emitVerbose, setEmitVerbose] = createSignal(true);
    const [emitDebug, setEmitDebug] = createSignal(true);
    const [logs, setLogs] = createSignal<string[]>([]);

    listen('source-wrench-log', (event) => {
        const logEvent = event.payload as LogEvent;
        if (!emitVerbose() && logEvent.level === 'Verbose') {
            return;
        }
        if (!emitDebug() && logEvent.level === 'Debug') {
            return;
        }
        setLogs([...logs(), `[${logEvent.level.toUpperCase()}] ${logEvent.message}`]);
    });

    return (
        <div class="dark:text-white w-screen absolute bottom-0 p-2 bg-white border border-gray-200 shadow dark:bg-gray-800 dark:border-gray-700">
            <div class="flex flex-row">
                <div class="flex items-center mb-4 mr-4">
                    <input
                        id="default-checkbox"
                        type="checkbox"
                        checked={emitVerbose()}
                        onChange={(event) => setEmitVerbose(event.target.checked)}
                        class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
                    />
                    <label for="default-checkbox" class="ms-2 text-sm font-medium text-gray-900 dark:text-gray-300">
                        Verbose
                    </label>
                </div>
                <div class="flex items-center mb-4">
                    <input
                        id="default-checkbox"
                        type="checkbox"
                        checked={emitDebug()}
                        onChange={(event) => setEmitDebug(event.target.checked)}
                        class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
                    />
                    <label for="default-checkbox" class="ms-2 text-sm font-medium text-gray-900 dark:text-gray-300">
                        Debug
                    </label>
                </div>
            </div>
            <ul class='overflow-y-auto max-h-36'>
                <For each={logs()}>{(log) => <li>{log}</li>}</For>
            </ul>
        </div>
    );
};

export default SourceFileSelector;
