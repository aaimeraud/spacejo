// Mentions légales. SpaceJo est un projet personnel et scolaire, open source et non commercial, édité à titre non
// professionnel : la loi (LCEN, art. 1-1 II) permet alors à l'éditeur de rester anonyme en ne publiant que
// l'identité et l'adresse de l'hébergeur, à condition de lui avoir donné ses propres coordonnées.
// Décision et sources : docs/decisions/2026-11-04-mentions-legales.md.
import { GITHUB_REPO } from "../config";
import { WizardFrame } from "./wizard/RuleSteps";

const REPO_URL = `https://github.com/${GITHUB_REPO}`;
const ISSUES_URL = `${REPO_URL}/issues`;
/** Adresse de contact choisie par l'éditeur pour être joint (ce n'est pas une coordonnée personnelle imposée). */
const CONTACT_EMAIL = "danielfortune.pro@proton.me";

export function LegalNotice({ onClose }: { onClose: () => void }) {
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
          <h3>Données traitées</h3>
          <p>
            Aucun compte n'est demandé. Pour faire fonctionner une partie, le serveur enregistre : le nom choisi
            pour jouer (un pseudonyme suffit), l'état de la partie, les messages du chat et une empreinte du jeton
            secret qui identifie chaque joueur. Ces données servent uniquement au jeu, ne sont ni vendues ni
            partagées, et ne font l'objet d'aucune mesure d'audience.
          </p>
          <p>
            Votre adresse IP sert, en mémoire seulement, à limiter le nombre de requêtes pour protéger le service
            contre les abus : le serveur de SpaceJo ne l'enregistre ni dans la base ni dans ses journaux.
            L'hébergeur peut la conserver dans ses propres journaux techniques.
          </p>
          <p>
            Les messages du chat et leurs réactions ne sont pas conservés : ils sont effacés 15 minutes après la fin
            de la partie, ou après 24 heures sans aucun coup si la partie a été abandonnée. Les parties elles-mêmes ne
            sont pas encore supprimées automatiquement. Vous pouvez demander l'accès à vos données,
            leur rectification ou leur suppression en écrivant à{" "}
            <a href={`mailto:${CONTACT_EMAIL}`}>{CONTACT_EMAIL}</a> (indiquez le code de la partie et le nom utilisé).
            Vous pouvez aussi saisir la CNIL (cnil.fr).
          </p>
        </section>

        <section>
          <h3>Stockage dans le navigateur</h3>
          <p>
            Aucun cookie. Le site garde seulement dans votre navigateur (stockage local) ce qu'il lui faut pour
            fonctionner : votre jeton de joueur pour reprendre une partie, la liste des parties rejointes sur cet
            appareil, le thème choisi et, pendant 10 minutes, la dernière version publiée sur GitHub. Ces données
            restent sur votre appareil ; les effacer depuis votre navigateur vous fait seulement perdre l'accès à vos
            parties en cours.
          </p>
        </section>

        <section>
          <h3>Services tiers</h3>
          <p>
            Votre navigateur contacte directement Google Fonts (police Inter) et l'API publique de GitHub (dernière
            version et dernier commit affichés sur l'accueil). Ces services reçoivent donc votre adresse IP, selon
            leurs propres règles de confidentialité.
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
