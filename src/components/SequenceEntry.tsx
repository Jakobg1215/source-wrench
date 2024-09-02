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
        <div class="Sequence-Entry">
            <h3>Sequence</h3>
            <label>
                Name:
                <input name="SequenceName" type="text" value={properties.data.name} onChange={(event) => changeSequenceName(event.target.value)} />
            </label>
            <br />
            <label>
                Animations:
                <br />
                <For each={grid()}>
                    {(row, rowIndex) => (
                        <div>
                            <For each={row}>
                                {(column, columnIndex) => (
                                    <input
                                        name={`SequenceAnimation${rowIndex()}${columnIndex()}`}
                                        list="Animation-Names"
                                        value={column}
                                        onChange={(event) => updateGridValue(rowIndex(), columnIndex(), event.target.value)}
                                    ></input>
                                )}
                            </For>
                            <button onClick={() => addColumn()}>+</button>
                            <button onClick={() => removeColumn()}>-</button>
                        </div>
                    )}
                </For>
                <button onClick={() => addRow()}>+</button>
                <button onClick={() => removeRow()}>-</button>
                <br />
            </label>
            <br />
            <button onClick={() => removeSequence()}>Remove</button>
        </div>
    );
};

export default SequenceEntry;
