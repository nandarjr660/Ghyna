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
    font-family: var(--font-ui);
  }
  .header {
    flex: none;
    display: flex;
    align-items: center;
    gap: 10px;
    height: 22px;
  }
  .title {
    font-family: var(--font-display);
    font-size: 15px;
    font-weight: 600;
    letter-spacing: 0.015em;
    color: var(--accent);
  }
  .count {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 9.5px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
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
    height: 22px;
    padding: 0 9px;
    border: 0;
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.06);
    box-shadow: inset 0 0 0 0.5px var(--line);
    color: var(--muted);
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    transition:
      background var(--motion-ui) var(--ease-out),
      color var(--motion-ui) var(--ease-out),
      transform var(--motion-fast) var(--ease-out);
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
    background: rgba(255, 255, 255, 0.13);
    color: var(--text);
  }
  .add:active, .save:active, .check:active, .remove:active { transform: scale(.97); }

  .progress {
    flex: none;
    height: 2px;
    margin-top: 12px;
    overflow: hidden;
    border-radius: 2px;
    background: rgba(255, 255, 255, 0.13);
  }
  .progress i {
    background: var(--accent);
    transition: width 220ms var(--ease-out);
  }

  .form {
    flex: none;
    display: flex;
    gap: 6px;
    height: 28px;
    margin-top: 9px;
    animation: form-in var(--motion-ui) var(--ease-out) both;
  }
  .form input {
    min-width: 0;
    height: 28px;
    padding: 0 9px;
    border: 0;
    border-radius: var(--radius-sm);
    background: rgba(255, 255, 255, 0.05);
    box-shadow: inset 0 0 0 0.5px var(--line);
    color: var(--text);
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
    font-family: var(--font-mono);
    font-size: 10px;
    font-variant-numeric: tabular-nums;
  }
  .form input:focus {
    outline: 1.5px solid var(--accent-line);
    outline-offset: 0;
  }
  .save {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    padding: 0;
    border-radius: var(--radius-xs);
    background: var(--accent);
    color: var(--accent-ink);
    box-shadow: inset 0 0 0 0.5px rgba(255, 255, 255, 0.24);
    transition: background var(--motion-ui) var(--ease-out), transform var(--motion-fast) var(--ease-out);
  }
  .save:hover {
    background: var(--accent-strong);
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
    border-top: 1px solid var(--line);
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
    border: 1.5px solid rgba(255, 255, 255, 0.28);
    border-radius: 50%;
    transition:
      background var(--motion-ui) var(--ease-out),
      border-color var(--motion-ui) var(--ease-out),
      transform var(--motion-fast) var(--ease-out);
  }
  .check:hover {
    border-color: var(--accent-line);
  }
  .check svg {
    width: 9px;
    height: 9px;
    fill: none;
    stroke: var(--accent-ink);
    stroke-width: 2.6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .done .check {
    border-color: var(--accent);
    background: var(--accent);
  }
  .label {
    flex: 1;
    min-width: 0;
    padding: 0;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: rgba(238, 246, 248, 0.88);
    transition: color var(--motion-ui) var(--ease-out);
  }
  .label:hover {
    color: var(--text);
  }
  .done .label {
    color: rgba(238, 246, 248, 0.36);
    text-decoration: line-through;
  }
  .time-chip {
    flex: none;
    display: flex;
    align-items: center;
    gap: 4px;
    font-family: var(--font-mono);
    font-size: 9.5px;
    letter-spacing: -0.01em;
    color: var(--muted);
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
    border-radius: var(--radius-xs);
    color: var(--muted);
    opacity: 0;
    transition:
      opacity 150ms var(--ease-out),
      color 150ms var(--ease-out),
      background 150ms var(--ease-out),
      transform var(--motion-fast) var(--ease-out);
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
    background: rgba(255, 255, 255, 0.12);
    color: var(--text);
  }

  .empty {
    display: flex;
    flex-direction: column;
    justify-content: center;
    min-height: 66px;
    gap: 4px;
    border-top: 0;
    color: var(--muted-strong);
    font-size: 11.5px;
  }
  .empty small {
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.02em;
    color: var(--muted);
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  @keyframes form-in {
    from { opacity: 0; transform: translateY(-3px); }
    to { opacity: 1; transform: translateY(0); }
  }
  @media (prefers-reduced-motion: reduce) {
    .add,
    .progress i,
    .save,
    .check,
    .remove {
      transition: none;
      animation: none;
      transform: none !important;
    }
  }
</style>
