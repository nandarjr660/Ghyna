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
  const pct = $derived(total ? Math.round((done / total) * 100) : 0);

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
    <span class="title">Tugas</span>
    <span class="count">{done}/{total}</span>
    {#if editing === null}
      <button class="add" onclick={() => edit()} aria-label="Tambah tugas">
        Tambah
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>
      </button>
    {/if}
  </div>

  {#if total > 0}
    <div class="progress" aria-hidden="true"><i style:width="{pct}%"></i></div>
  {/if}

  {#if editing !== null}
    <div class="form" onfocusout={blur}>
      <input bind:this={input} bind:value={text} onkeydown={keys} placeholder="Tugas baru…" class="text" aria-label="Nama tugas" />
      <input bind:value={time} onkeydown={keys} type="time" class="time" title="Ingatkan jam (opsional)" aria-label="Jam pengingat" />
      <button class="save" onclick={() => finish(true)} aria-label="Simpan tugas">
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m5 12 4 4L19 6" /></svg>
      </button>
    </div>
  {/if}

  <ul class="scroll">
    {#each todo.tasks.filter((t) => t.id !== editing) as task (task.id)}
      <li class:done={task.done}>
        <button class="check" onclick={() => todo.toggle(task.id)} aria-label={task.done ? "Belum selesai" : "Selesai"}>
          {#if task.done}<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m5 12 4 4L19 6" /></svg>{/if}
        </button>
        <button class="label" onclick={() => edit(task)}>{task.text}</button>
        {#if task.time}
          <span class="time-chip">
            <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="8" /><path d="M12 7v5l3 2" /></svg>
            {task.time}
          </span>
        {/if}
        <button class="remove" onclick={() => todo.remove(task.id)} aria-label="Hapus">
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18" /></svg>
        </button>
      </li>
    {:else}
      {#if editing === null}
        <li class="empty">
          <span>Belum ada tugas</span>
          <small>Tambah satu, bisa dengan jam pengingat.</small>
        </li>
      {/if}
    {/each}
  </ul>
</div>

<style>
  .page {
    position: absolute;
    inset: 0;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    padding: 10px 42px 19px;
    overflow: hidden;
  }
  .header {
    flex: none;
    display: flex;
    align-items: center;
    gap: 10px;
    height: 22px;
  }
  .title {
    font-size: 15px;
    font-weight: 600;
  }
  .count {
    flex: 1;
    font-size: 12.5px;
    color: rgba(255, 255, 255, 0.42);
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
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 24px;
    padding: 0 10px;
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: 8px;
    color: rgba(255, 255, 255, 0.52);
    font-size: 11px;
    font-weight: 500;
    transition:
      border-color 0.18s,
      color 0.18s,
      background 0.18s;
  }
  .add svg {
    width: 10px;
    height: 10px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
  }
  .add:hover {
    border-color: rgba(255, 255, 255, 0.38);
    background: rgba(255, 255, 255, 0.07);
    color: #fff;
  }

  .progress {
    flex: none;
    height: 2.5px;
    margin-top: 12px;
    overflow: hidden;
    border-radius: 1px;
    background: rgba(255, 255, 255, 0.12);
  }
  .progress i {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: #fff;
    transition: width 0.3s ease;
  }

  .form {
    flex: none;
    display: flex;
    gap: 6px;
    height: 28px;
    margin-top: 9px;
  }
  .form input {
    min-width: 0;
    height: 28px;
    padding: 0 9px;
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.04);
    color: #fff;
    font: inherit;
    font-size: 11px;
    color-scheme: dark;
  }
  .form .text {
    flex: 1;
  }
  .form .time {
    width: 78px;
    padding: 0 4px;
    font-size: 10px;
  }
  .form input:focus {
    outline: 1px solid rgba(255, 255, 255, 0.55);
    outline-offset: 0;
  }
  .save {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    padding: 0;
    border-radius: 7px;
    background: #fff;
    color: #000;
    transition: background 0.18s;
  }
  .save:hover {
    background: rgba(255, 255, 255, 0.86);
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

  ul {
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
    gap: 9px;
    min-height: 26px;
    font-size: 12px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
  }
  li:first-child {
    border-top: 0;
  }
  .check {
    flex: none;
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    padding: 0;
    border: 1.5px solid rgba(255, 255, 255, 0.35);
    border-radius: 50%;
    transition:
      background 0.18s,
      border-color 0.18s;
  }
  .check:hover {
    border-color: rgba(255, 255, 255, 0.78);
  }
  .check svg {
    width: 9px;
    height: 9px;
    fill: none;
    stroke: #000;
    stroke-width: 2.6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .done .check {
    border-color: #fff;
    background: #fff;
  }
  .label {
    flex: 1;
    min-width: 0;
    padding: 0;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: rgba(255, 255, 255, 0.88);
  }
  .label:hover {
    color: #fff;
  }
  .done .label {
    color: rgba(255, 255, 255, 0.4);
    text-decoration: line-through;
  }
  .time-chip {
    flex: none;
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 10.5px;
    color: rgba(255, 255, 255, 0.42);
    font-variant-numeric: tabular-nums;
  }
  .time-chip svg {
    width: 9px;
    height: 9px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
  }
  .remove {
    flex: none;
    display: grid;
    place-items: center;
    width: 18px;
    height: 18px;
    padding: 0;
    border-radius: 5px;
    color: rgba(255, 255, 255, 0.36);
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
    background: rgba(255, 255, 255, 0.09);
    color: #fff;
  }

  .empty {
    display: flex;
    flex-direction: column;
    justify-content: center;
    min-height: 66px;
    gap: 4px;
    border-top: 0;
    color: rgba(255, 255, 255, 0.62);
    font-size: 12px;
  }
  .empty small {
    font-size: 10px;
    color: rgba(255, 255, 255, 0.34);
  }

  button:focus-visible {
    outline: 1.5px solid rgba(255, 255, 255, 0.8);
    outline-offset: 2px;
  }
  @media (prefers-reduced-motion: reduce) {
    .add,
    .progress i,
    .save,
    .check,
    .remove {
      transition: none;
    }
  }
</style>
