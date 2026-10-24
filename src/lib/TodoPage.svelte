<script lang="ts">
  import { tick } from "svelte";
  import type { Task, Todo } from "./productivity.svelte";

  // `onTyping` asks Rust for keyboard focus (the notch normally never takes it).
  let { todo, onTyping }: { todo: Todo; onTyping: (typing: boolean) => void } = $props();

  // null: not editing; 0: a new task; otherwise the id being edited.
  let editing = $state<number | null>(null);
  let text = $state("");
  let time = $state("");
  let input = $state<HTMLInputElement>();

  const total = $derived(todo.tasks.length);
  const done = $derived(todo.doneToday());
  const segments = $derived(Math.min(Math.max(total, 1), 8));
  const filled = $derived(total ? Math.round((done / total) * segments) : 0);

  async function edit(task?: Task) {
    editing = task?.id ?? 0;
    text = task?.text ?? "";
    time = task?.time ?? "";
    onTyping(true);
    await tick();
    // Give Windows a moment to hand the notch keyboard focus.
    setTimeout(() => input?.focus(), 80);
  }

  function finish(keep: boolean) {
    if (keep && editing === 0) todo.add(text, time);
    else if (keep && editing) todo.update(editing, text, time);
    editing = null;
    onTyping(false);
  }

  function keys(e: KeyboardEvent) {
    if (e.key === "Enter") finish(true);
    if (e.key === "Escape") finish(false);
  }

  // Leaving the form (click elsewhere) saves what was typed.
  function blur(e: FocusEvent) {
    const next = e.relatedTarget as Node | null;
    if (!(e.currentTarget as HTMLElement).contains(next)) finish(text.trim() !== "");
  }
</script>

<div class="page">
  <div class="header">
    <span class="mark" aria-hidden="true">
      <svg viewBox="0 0 24 24"><path d="M5 4h14v16H5zM8 9h8M8 13h5" /><path d="m8 17 1.6 1.6L13 15" /></svg>
    </span>
    <span class="title">To-do</span>
    <span class="count">{done}/{total} selesai</span>
    {#if editing === null}
      <button class="add" onclick={() => edit()} aria-label="Tambah tugas">
        <svg viewBox="0 0 24 24"><path d="M12 5v14M5 12h14" /></svg>
      </button>
    {/if}
  </div>

  {#if total > 0}
    <div class="progress" aria-hidden="true">
      {#each Array.from({ length: segments }) as _, i}
        <i class:done={i < filled}></i>
      {/each}
    </div>
  {/if}

  {#if editing !== null}
    <div class="form" onfocusout={blur}>
      <input bind:this={input} bind:value={text} onkeydown={keys} placeholder="Tugas baru…" class="text" aria-label="Nama tugas" />
      <input bind:value={time} onkeydown={keys} type="time" class="time" title="Ingatkan jam (opsional)" aria-label="Jam pengingat" />
      <button class="save" onclick={() => finish(true)} aria-label="Simpan tugas">
        <svg viewBox="0 0 24 24"><path d="m5 12 4 4L19 6" /></svg>
      </button>
    </div>
  {/if}

  <ul class="scroll">
    {#each todo.tasks.filter((t) => t.id !== editing) as task (task.id)}
      <li class:done={task.done}>
        <button class="check" onclick={() => todo.toggle(task.id)} aria-label={task.done ? "Belum selesai" : "Selesai"}>
          {#if task.done}<svg viewBox="0 0 24 24"><path d="m5 12 4 4L19 6" /></svg>{/if}
        </button>
        <button class="label" onclick={() => edit(task)}>{task.text}</button>
        {#if task.time}
          <span class="chip">
            <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="8" /><path d="M12 7v5l3 2" /></svg>
            {task.time}
          </span>
        {/if}
        <button class="remove" onclick={() => todo.remove(task.id)} aria-label="Hapus">
          <svg viewBox="0 0 24 24"><path d="M6 6l12 12M18 6 6 18" /></svg>
        </button>
      </li>
    {:else}
      {#if editing === null}
        <li class="empty">
          <span class="empty-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24"><path d="M5 4h14v16H5zM8 9h8M8 13h5" /><path d="m8 17 1.6 1.6L13 15" /></svg>
          </span>
          <span>Belum ada tugas<small>Tambah satu, bisa dengan jam pengingat.</small></span>
        </li>
      {/if}
    {/each}
  </ul>
</div>

<style>
  .page {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    padding: 8px 42px 18px;
    overflow: hidden;
    --accent: #34d399;
    --accent-soft: rgba(52, 211, 153, 0.16);
    --accent-line: rgba(52, 211, 153, 0.4);
  }
  .page::before {
    content: "";
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: radial-gradient(150px 90px at 16% 12%, rgba(52, 211, 153, 0.13), transparent 70%);
  }

  .header {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 7px;
    height: 20px;
  }
  .mark {
    display: grid;
    place-items: center;
    width: 18px;
    height: 18px;
    border: 0.5px solid var(--accent-line);
    border-radius: 6px;
    background: var(--accent-soft);
    color: var(--accent);
  }
  .mark svg {
    width: 11px;
    height: 11px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .title {
    font-size: 12.5px;
    font-weight: 650;
    letter-spacing: 0.01em;
  }
  .count {
    flex: 1;
    font-size: 9.5px;
    color: rgba(255, 255, 255, 0.45);
    font-variant-numeric: tabular-nums;
  }
  button {
    border: 0;
    background: none;
    color: #fff;
    font: inherit;
    cursor: pointer;
  }
  .add {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border: 0.5px solid var(--accent-line);
    border-radius: 50%;
    background: var(--accent-soft);
    color: var(--accent);
    transition:
      background 0.2s,
      transform 0.18s;
  }
  .add svg {
    width: 11px;
    height: 11px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2.2;
    stroke-linecap: round;
  }
  .add:hover {
    background: rgba(52, 211, 153, 0.3);
    transform: rotate(90deg);
  }
  .add:active {
    transform: scale(0.92);
  }

  .progress {
    position: relative;
    z-index: 1;
    display: flex;
    gap: 3px;
    height: 3px;
    margin-top: 5px;
  }
  .progress i {
    flex: 1;
    border-radius: 2px;
    background: rgba(255, 255, 255, 0.09);
    transition: background 0.3s;
  }
  .progress i.done {
    background: var(--accent);
    box-shadow: 0 0 6px rgba(52, 211, 153, 0.4);
  }

  .form {
    position: relative;
    z-index: 2;
    display: flex;
    gap: 4px;
    margin-top: 5px;
    height: 24px;
  }
  .form input {
    min-width: 0;
    height: 24px;
    padding: 0 7px;
    border: 0.5px solid rgba(255, 255, 255, 0.16);
    border-radius: 7px;
    background: rgba(255, 255, 255, 0.06);
    color: #fff;
    font: inherit;
    font-size: 10.5px;
    color-scheme: dark;
  }
  .form .text {
    flex: 1;
  }
  .form .time {
    width: 74px;
    padding: 0 4px;
    font-size: 9.5px;
  }
  .form input:focus {
    outline: 1.5px solid var(--accent-line);
    outline-offset: 0;
  }
  .save {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 7px;
    background: var(--accent);
    color: #04120c;
    transition: transform 0.18s;
  }
  .save svg {
    width: 12px;
    height: 12px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2.4;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .save:hover {
    transform: translateY(-1px);
  }
  .save:active {
    transform: scale(0.95);
  }

  ul {
    position: relative;
    z-index: 1;
    flex: 1;
    min-height: 0;
    margin: 5px 0 0;
    padding: 0;
    list-style: none;
    overflow-y: auto;
    scrollbar-width: none;
  }
  li {
    display: flex;
    align-items: center;
    gap: 7px;
    min-height: 18px;
    font-size: 11.5px;
  }
  .check {
    flex: none;
    display: grid;
    place-items: center;
    width: 14px;
    height: 14px;
    padding: 0;
    border: 1px solid rgba(255, 255, 255, 0.3);
    border-radius: 50%;
    transition:
      background 0.2s,
      border-color 0.2s,
      transform 0.18s;
  }
  .check:hover {
    border-color: var(--accent);
  }
  .check svg {
    width: 9px;
    height: 9px;
    fill: none;
    stroke: #04120c;
    stroke-width: 2.6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .done .check {
    border-color: var(--accent);
    background: var(--accent);
    transform: scale(1.02);
  }
  .label {
    flex: 1;
    min-width: 0;
    padding: 0;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .label:hover {
    color: var(--accent);
  }
  .done .label {
    color: rgba(255, 255, 255, 0.38);
    text-decoration: line-through;
  }
  .chip {
    flex: none;
    display: flex;
    align-items: center;
    gap: 3px;
    padding: 1px 5px;
    border: 0.5px solid rgba(251, 191, 36, 0.22);
    border-radius: 999px;
    background: rgba(251, 191, 36, 0.1);
    color: #fcd34d;
    font-size: 8.5px;
    font-variant-numeric: tabular-nums;
  }
  .chip svg {
    width: 8px;
    height: 8px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
  }
  .remove {
    flex: none;
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    padding: 0;
    border-radius: 4px;
    color: rgba(255, 255, 255, 0.35);
    opacity: 0;
    transition:
      opacity 0.15s,
      color 0.15s,
      background 0.15s;
  }
  .remove svg {
    width: 9px;
    height: 9px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
  }
  li:hover .remove,
  .remove:focus-visible {
    opacity: 1;
  }
  .remove:hover {
    background: rgba(248, 113, 113, 0.14);
    color: #f87171;
  }

  .empty {
    gap: 8px;
    margin-top: 4px;
    color: rgba(255, 255, 255, 0.55);
    font-size: 10.5px;
  }
  .empty-icon {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border: 0.5px dashed var(--accent-line);
    border-radius: 8px;
    color: var(--accent);
  }
  .empty-icon svg {
    width: 13px;
    height: 13px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .empty small {
    display: block;
    margin-top: 1px;
    font-size: 8.5px;
    color: rgba(255, 255, 255, 0.35);
  }

  button:focus-visible {
    outline: 1.5px solid var(--accent);
    outline-offset: 2px;
  }
  @media (prefers-reduced-motion: reduce) {
    .add,
    .save,
    .check,
    .progress i {
      transition: none;
    }
  }
</style>
