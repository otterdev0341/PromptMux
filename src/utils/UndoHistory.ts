export class UndoHistory<T> {
    private past: T[] = [];
    private future: T[] = [];
    private maxHistory: number;

    constructor(maxHistory = 50) {
        this.maxHistory = maxHistory;
    }

    /**
     * Pushes a new state into the history.
     * Clears the future stack since a new branch of history is created.
     * @param currentState The state BEFORE the change (or the new state? usually we push the NEW state, but standard patterns vary)
     * Correction: Usually we want to safeguard the CURRENT state before it changes, OR we simply store snapshots.
     * Simple snapshot approach:
     * - push(newState) -> adds to past.
     * - undo() -> pops from past, adds current to future, returns previous.
     */
    public push(state: T): void {
        // If the new state is identical to the last state, don't push (deduplicate)
        if (this.past.length > 0 && JSON.stringify(this.past[this.past.length - 1]) === JSON.stringify(state)) {
            return;
        }

        this.past.push(state);
        if (this.past.length > this.maxHistory) {
            this.past.shift();
        }
        this.future = []; // Clear redo stack on new action
    }

    public get canUndo(): boolean {
        return this.past.length > 0;
    }

    public get canRedo(): boolean {
        return this.future.length > 0;
    }

    public undo(currentState: T): T | null {
        if (!this.canUndo) return null;

        // Check if the latest snapshot equals current state (snapshot model)
        // If we are "on" the tip, pop it to move "behind" it
        let tip = this.past[this.past.length - 1];
        if (JSON.stringify(tip) === JSON.stringify(currentState)) {
            const currentSnapshot = this.past.pop();
            if (currentSnapshot !== undefined) {
                this.future.push(currentSnapshot);
            }
        }

        if (this.past.length === 0) {
            // Cannot undo further
            return null;
        }

        // Return the new tip
        const previous = this.past[this.past.length - 1];
        return previous;
    }

    public redo(currentState: T): T | null {
        if (!this.canRedo) return null;

        const next = this.future.pop();
        if (next === undefined) return null;

        this.past.push(next);
        return next;
    }

    public clear(): void {
        this.past = [];
        this.future = [];
    }
}
