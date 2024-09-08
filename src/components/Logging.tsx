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
        <section id="Logging-Menu" class='bg-orange-400'>
            <h2>Log</h2>
            <label>
                Verbose
                <input name="Verbose" type="checkbox" checked={true} onChange={(event) => setEmitVerbose(event.target.checked)}></input>
            </label>
            <label>
                Debug
                <input name="Debug" type="checkbox" checked={true} onChange={(event) => setEmitDebug(event.target.checked)}></input>
            </label>
            <ul>
                <For each={logs()}>{(log) => <li>{log}</li>}</For>
            </ul>
        </section>
    );
};

export default SourceFileSelector;
