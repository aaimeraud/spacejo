// Bas de l'accueil : dernière version publiée (tag de la release) et dernier commit de `main`, lus sur GitHub
// (github.ts). Chaque élément est un lien vers sa page GitHub ; rien ne s'affiche tant que GitHub n'a pas répondu.
import { GitCommitHorizontal, Tag } from "lucide-react";
import { useEffect, useState } from "react";
import { loadRepoStatus, type RepoStatus } from "../github";
import type { KeyValueStorage } from "../session";

/** `localStorage`, ou un stockage vide si le navigateur l'interdit (son simple accès peut lever une erreur). */
function browserStorage(): KeyValueStorage {
  try {
    return window.localStorage;
  } catch {
    return { getItem: () => null, setItem: () => undefined, removeItem: () => undefined };
  }
}

export function RepoTrackers() {
  const [status, setStatus] = useState<RepoStatus | null>(null);

  useEffect(() => {
    let cancelled = false;
    void loadRepoStatus((url) => fetch(url), browserStorage(), Date.now()).then((loaded) => {
      if (!cancelled) setStatus(loaded);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  if (status === null || (status.release === null && status.commit === null)) return null;
  return (
    <ul className="repo-trackers" aria-label="Version du jeu sur GitHub">
      {status.release !== null && (
        <li>
          <a href={status.release.url} target="_blank" rel="noopener noreferrer">
            <Tag aria-hidden="true" /> Version {status.release.tag}
          </a>
        </li>
      )}
      {status.commit !== null && (
        <li>
          <a
            href={status.commit.url}
            target="_blank"
            rel="noopener noreferrer"
            title={status.commit.title}
            aria-label={`Dernier commit ${status.commit.shortSha} : ${status.commit.title}`}
          >
            <GitCommitHorizontal aria-hidden="true" /> Dernier commit <code>{status.commit.shortSha}</code>
          </a>
        </li>
      )}
    </ul>
  );
}
