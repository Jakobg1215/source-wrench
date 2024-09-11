import { createSignal, For, type Component } from 'solid-js';
import { type SetStoreFunction } from 'solid-js/store';

type SequenceEntryProperties = {
    readonly identifier: number;
    readonly setSequenceEntries: SetStoreFunction<SequenceEntryProperties[]>;
    readonly data: {
        name: string;
        animations: string[][];
    };
};

export type { SequenceEntryProperties };
// TODO: This code is bad and should be refactored.
const SequenceEntry: Component<SequenceEntryProperties> = (properties) => {
    const [grid, setGrid] = createSignal([['']]);

    const removeSequence = () => {
        properties.setSequenceEntries((sequences) => sequences.filter((sequence) => sequence.identifier !== properties.identifier));
    };

    const changeSequenceName = (name: string) => {
        properties.setSequenceEntries((sequence) => sequence.identifier == properties.identifier, 'data', 'name', name);
    };

    const addRow = () => {
        setGrid([...grid(), new Array(grid()[0]?.length).fill('')]);
        properties.setSequenceEntries((sequence) => sequence.identifier == properties.identifier, 'data', 'animations', grid());
    };

    const removeRow = () => {
        if (grid().length > 1) {
            setGrid(grid().slice(0, -1));
            properties.setSequenceEntries((sequence) => sequence.identifier == properties.identifier, 'data', 'animations', grid());
        }
    };

    const addColumn = () => {
        setGrid(grid().map((row) => [...row, '']));
        properties.setSequenceEntries((sequence) => sequence.identifier == properties.identifier, 'data', 'animations', grid());
    };

    const removeColumn = () => {
        if (grid()[0]?.length! > 1) {
            setGrid(grid().map((row) => row.slice(0, -1)));
            properties.setSequenceEntries((sequence) => sequence.identifier == properties.identifier, 'data', 'animations', grid());
        }
    };

    const updateGridValue = (rowIndex: number, columnIndex: number, newValue: string) => {
        setGrid(grid().map((row, rIndex) => row.map((cell, cIndex) => (rIndex === rowIndex && cIndex === columnIndex ? newValue : cell))));
        properties.setSequenceEntries((sequence) => sequence.identifier == properties.identifier, 'data', 'animations', grid());
    };

    return (
        <div class="max-w-md p-6 bg-white border border-gray-200 rounded-lg shadow dark:bg-gray-800 dark:border-gray-700 ">
            <h5 class="mb-2 text-2xl font-bold tracking-tight text-gray-900 dark:text-white">Sequence</h5>
            <div>
                <label for="first_name" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">
                    Name
                </label>
                <input
                    type="text"
                    id="first_name"
                    value={properties.data.name}
                    onChange={(event) => changeSequenceName(event.target.value)}
                    class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
                    placeholder="idkman"
                    required
                />
            </div>
            <br />
            <label>
                <p class="text-sm font-medium text-gray-900 dark:text-white">Animations:</p>
                <br />
                <For each={grid()}>
                    {(row, rowIndex) => (
                        <div class='mt-2'>
                            <For each={row}>
                                {(column, columnIndex) => (
                                    <div>
                                        <input
                                            name={`SequenceAnimation${rowIndex()}${columnIndex()}`}
                                            list="Animation-Names"
                                            value={column}
                                            onChange={(event) => updateGridValue(rowIndex(), columnIndex(), event.target.value)}
                                            type="text"
                                            class="bg-gray-50 border mb-2 border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
                                            placeholder="idkman"
                                            required
                                        />
                                    </div>
                                )}
                            </For>

                            <div class="inline-flex rounded-md shadow-sm mt-2" role="group">
                                <button
                                    onClick={() => addColumn()}
                                    type="button"
                                    class="px-4 py-2 text-sm font-medium text-gray-900 bg-white border border-gray-200 rounded-s-lg hover:bg-gray-100 hover:text-blue-700 focus:z-10 focus:ring-2 focus:ring-blue-700 focus:text-blue-700 dark:bg-gray-800 dark:border-gray-700 dark:text-white dark:hover:text-white dark:hover:bg-gray-700 dark:focus:ring-blue-500 dark:focus:text-white"
                                >
                                    Add Column
                                </button>
                                <button
                                    onClick={() => removeColumn()}
                                    type="button"
                                    class="px-4 py-2 text-sm font-medium text-gray-900 bg-white border border-gray-200 rounded-e-lg hover:bg-gray-100 hover:text-blue-700 focus:z-10 focus:ring-2 focus:ring-blue-700 focus:text-blue-700 dark:bg-gray-800 dark:border-gray-700 dark:text-white dark:hover:text-white dark:hover:bg-gray-700 dark:focus:ring-blue-500 dark:focus:text-white"
                                >
                                    Remove Column
                                </button>
                            </div>
                        </div>
                    )}
                </For>
                <div class="inline-flex rounded-md shadow-sm mt-2" role="group">
                    <button
                        onClick={() => addRow()}
                        type="button"
                        class="px-4 py-2 text-sm font-medium text-gray-900 bg-white border border-gray-200 rounded-s-lg hover:bg-gray-100 hover:text-blue-700 focus:z-10 focus:ring-2 focus:ring-blue-700 focus:text-blue-700 dark:bg-gray-800 dark:border-gray-700 dark:text-white dark:hover:text-white dark:hover:bg-gray-700 dark:focus:ring-blue-500 dark:focus:text-white"
                    >
                        Add Row
                    </button>
                    <button
                        onClick={() => removeRow()}
                        type="button"
                        class="px-4 py-2 text-sm font-medium text-gray-900 bg-white border border-gray-200 rounded-e-lg hover:bg-gray-100 hover:text-blue-700 focus:z-10 focus:ring-2 focus:ring-blue-700 focus:text-blue-700 dark:bg-gray-800 dark:border-gray-700 dark:text-white dark:hover:text-white dark:hover:bg-gray-700 dark:focus:ring-blue-500 dark:focus:text-white"
                    >
                        Remove Row
                    </button>
                </div>
                <br />
            </label>
            <br />
            <button
                class="text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 me-2 mb-2 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none dark:focus:ring-blue-800"
                onClick={() => removeSequence()}
            >
                Remove
            </button>
        </div>
    );
};

export default SequenceEntry;
