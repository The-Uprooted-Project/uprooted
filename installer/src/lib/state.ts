type Listener<T> = (value: T) => void;

export class State<T> {
  private value: T;
  private listeners: Listener<T>[] = [];
  constructor(initial: T) { this.value = initial; }
  get(): T { return this.value; }
  set(next: T): void {
    this.value = next;
    for (const fn of this.listeners) fn(next);
  }
  subscribe(fn: Listener<T>): () => void {
    this.listeners.push(fn);
    return () => { this.listeners = this.listeners.filter((l) => l !== fn); };
  }
}
