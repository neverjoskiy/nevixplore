import { FolderOpen } from "./icons";

interface ListPaneProps {
  path: string;
  onBrowse: () => void;
}

/**
 * The file list. PR-0 shows the column header and an empty state; PR-3 swaps
 * the body for a virtualized list and PR-4 feeds it from `list_dir`.
 *
 * The header is already a real table header so the column-width machinery
 * lands in one place rather than being retrofitted.
 */
export function ListPane({ path, onBrowse }: ListPaneProps) {
  return (
    <section className="list-pane" aria-label="Files">
      <div className="list-pane__header" role="presentation">
        <span className="col col--name">Name</span>
        <span className="col col--size">Size</span>
        <span className="col col--modified">Date modified</span>
        <span className="col col--kind">Kind</span>
      </div>

      <div className="list-pane__body">
        <div className="empty-state">
          <span className="empty-state__icon" aria-hidden="true">
            <FolderOpen size={28} />
          </span>
          <p className="empty-state__title">Nothing to show yet</p>
          <p className="empty-state__body">
            Listing <span className="empty-state__path truncate">{path}</span> arrives in the next
            milestone.
          </p>
          <button className="button" onClick={onBrowse}>
            Choose a folder
          </button>
        </div>
      </div>
    </section>
  );
}
