// Mentions légales. SpaceJo est un projet personnel et scolaire, open source et non commercial, édité à titre non
// professionnel : la loi (LCEN, art. 1-1 II) permet alors à l'éditeur de rester anonyme en ne publiant que
// l'identité et l'adresse de l'hébergeur, à condition de lui avoir donné ses propres coordonnées.
// Décision et sources : docs/decisions/2026-11-04-mentions-legales.md.
import { CONTACT_EMAIL, GITHUB_REPO } from "../config";
import { WizardFrame } from "./wizard/RuleSteps";

const REPO_URL = `https://github.com/${GITHUB_REPO}`;
const ISSUES_URL = `${REPO_URL}/issues`;

export function LegalNotice({ onClose, onPrivacy }: { onClose: () => void; onPrivacy: () => void }) {
  return (
    <WizardFrame index={0} total={1} title="Mentions légales" onBack={onClose}>
      <div className="legal-notice">
        <section>
          <h3>Éditeur</h3>
          <p>
            SpaceJo est un projet personnel et scolaire, open source et sans but lucratif : aucune publicité, aucun
            paiement, aucun revenu. Il est édité à titre non professionnel par une personne physique qui, comme
            le permet l'article 1-1 II de la loi n° 2004-575 du 21 juin 2004 pour la confiance dans l'économie
            numérique (LCEN), ne publie pas ses coordonnées : elles ont été communiquées à l'hébergeur ci-dessous.
          </p>
          <p>
            Contact : <a href={`mailto:${CONTACT_EMAIL}`}>{CONTACT_EMAIL}</a>, ou par une demande (« issue ») sur
            le dépôt{" "}
            <a href={ISSUES_URL} target="_blank" rel="noreferrer">
              github.com/{GITHUB_REPO}
            </a>
            .
          </p>
        </section>

        <section>
          <h3>Hébergement</h3>
          <p>
            Site et serveur : Render Inc., 525 Brannan Street, San Francisco, CA 94107, États-Unis —{" "}
            <a href="https://render.com" target="_blank" rel="noreferrer">
              render.com
            </a>
            .
          </p>
          <p>
            Base de données : Supabase Pte. Ltd., 65 Chulia Street #38-02/03, OCBC Centre, Singapour 049513 —{" "}
            <a href="https://supabase.com" target="_blank" rel="noreferrer">
              supabase.com
            </a>
            . Les données sont stockées en Europe (Irlande).
          </p>
        </section>

        <section>
          <h3>Données personnelles</h3>
          <p>
            Aucun compte n'est demandé, aucun cookie n'est utilisé, et le serveur supprime de lui-même les parties et
            leurs messages après un court délai. Le détail (données enregistrées, durées, prestataires, vos droits) est
            dans la{" "}
            <button type="button" className="link-button" onClick={onPrivacy}>
              politique de confidentialité
            </button>
            .
          </p>
        </section>

        <section>
          <h3>Propriété intellectuelle</h3>
          <p>
            Le code source est public sur{" "}
            <a href={REPO_URL} target="_blank" rel="noreferrer">
              GitHub
            </a>
            . Les visuels sont originaux. SpaceJo reprend les règles du jeu de cartes Skyjo, marque de Magilano :
            projet indépendant, non affilié à Magilano ni approuvé par elle.
          </p>
        </section>
      </div>
    </WizardFrame>
  );
}
