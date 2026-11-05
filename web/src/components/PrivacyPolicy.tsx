// Politique de confidentialité : l'information due aux joueurs sur leurs données personnelles (RGPD, art. 13), et le
// stockage dans le navigateur (loi Informatique et Libertés, art. 82). Elle décrit ce que fait le code ACTUEL : toute
// évolution des données enregistrées (classement, comptes…) doit mettre cette page à jour dans le même commit.
// Décision et sources : docs/decisions/2026-11-05-politique-de-confidentialite.md.
import {
  CHAT_KEPT_AFTER_GAME_OVER_MINUTES,
  CONTACT_EMAIL,
  GAME_KEPT_WHEN_IDLE_HOURS,
  GITHUB_CACHE_MS,
  MAX_KNOWN_SESSIONS,
} from "../config";
import { WizardFrame } from "./wizard/RuleSteps";

/** Date de la dernière mise à jour de cette page (à changer à chaque modification de son contenu). */
const UPDATED_ON = "5 novembre 2026";

/** Durée du cache de la version GitHub, en minutes. */
const GITHUB_CACHE_MINUTES = Math.round(GITHUB_CACHE_MS / 60_000);

const mail = <a href={`mailto:${CONTACT_EMAIL}`}>{CONTACT_EMAIL}</a>;

/** Lien externe, ouvert dans un nouvel onglet. */
function External({ href, children }: { href: string; children: string }) {
  return (
    <a href={href} target="_blank" rel="noreferrer">
      {children}
    </a>
  );
}

interface Row {
  data: string;
  purpose: string;
  basis: string;
  kept: string;
}

/** Données enregistrées par le serveur de SpaceJo, avec leur finalité, leur base légale et leur durée. */
const SERVER_DATA: Row[] = [
  {
    data: "Le nom choisi pour jouer (un pseudonyme suffit)",
    purpose: "Vous montrer aux autres joueurs de la partie, et dans la liste des parties publiques pour l'hôte",
    basis: "Intérêt légitime : faire fonctionner le jeu que vous avez demandé",
    kept: `${GAME_KEPT_WHEN_IDLE_HOURS} h après la dernière action de la partie`,
  },
  {
    data: "L'état de la partie : cartes, scores, réglages, tours manqués, départs",
    purpose: "Jouer la partie (c'est le serveur qui applique les règles)",
    basis: "Intérêt légitime : faire fonctionner le jeu",
    kept: `${GAME_KEPT_WHEN_IDLE_HOURS} h après la dernière action de la partie`,
  },
  {
    data: "L'empreinte (hachage SHA-256) de votre jeton secret de joueur, jamais le jeton lui-même",
    purpose: "Vérifier que c'est bien vous qui jouez votre tour",
    basis: "Intérêt légitime : sécurité de la partie",
    kept: `${GAME_KEPT_WHEN_IDLE_HOURS} h après la dernière action de la partie`,
  },
  {
    data: "L'heure de votre dernière requête (présence)",
    purpose: "Savoir si un joueur est absent, pour que la partie ne reste pas bloquée",
    basis: "Intérêt légitime : faire fonctionner le jeu",
    kept: "Supprimée avec la partie",
  },
  {
    data: "Les messages du chat (texte, nom de l'auteur, heure) et les réactions",
    purpose: "Discuter pendant la partie",
    basis: "Intérêt légitime : faire fonctionner le chat que vous utilisez",
    kept: `${CHAT_KEPT_AFTER_GAME_OVER_MINUTES} min après la fin de la partie (au plus tard avec la partie) : aucun historique`,
  },
  {
    data: "Votre adresse IP",
    purpose: "Limiter le nombre de requêtes par minute, pour protéger le service contre les abus",
    basis: "Intérêt légitime : sécurité du service",
    kept: "Jamais enregistrée : gardée seulement dans la mémoire vive du serveur, vidée à chaque redémarrage",
  },
];

export function PrivacyPolicy({ onClose }: { onClose: () => void }) {
  return (
    <WizardFrame index={0} total={1} title="Confidentialité" subtitle={`Mise à jour : ${UPDATED_ON}`} onBack={onClose}>
      <div className="legal-notice">
        <section>
          <h3>En bref</h3>
          <p>
            SpaceJo ne demande ni compte, ni adresse e-mail, ni vrai nom. Le serveur garde seulement ce qu'il faut pour
            jouer, et le supprime de lui-même : les messages du chat {CHAT_KEPT_AFTER_GAME_OVER_MINUTES} minutes après la
            fin de la partie, la partie entière {GAME_KEPT_WHEN_IDLE_HOURS} heures après sa dernière action. Pas de
            publicité, pas de mesure d'audience, pas de cookie, rien n'est vendu ni partagé.
          </p>
        </section>

        <section>
          <h3>Responsable du traitement</h3>
          <p>
            L'éditeur de SpaceJo, une personne physique qui publie ce projet personnel et scolaire à titre non
            professionnel (voir les mentions légales). Pour toute question sur vos données : {mail}.
          </p>
        </section>

        <section>
          <h3>Données enregistrées par le serveur</h3>
          {/* Une fiche par donnée plutôt qu'un tableau : quatre colonnes ne tiennent pas sur un téléphone. */}
          <ul className="legal-data">
            {SERVER_DATA.map((row) => (
              <li key={row.data}>
                <h4>{row.data}</h4>
                <dl>
                  <dt>Pourquoi</dt>
                  <dd>{row.purpose}</dd>
                  <dt>Base légale (RGPD, art. 6)</dt>
                  <dd>{row.basis}</dd>
                  <dt>Durée</dt>
                  <dd>{row.kept}</dd>
                </dl>
              </li>
            ))}
          </ul>
          <p>
            Les journaux du serveur de SpaceJo ne notent que la route appelée et le résultat (par exemple « POST
            /api/action → 200 ») : ni adresse IP, ni nom, ni jeton, ni code de partie, ni message.
          </p>
          <p>
            Le nom et les messages sont écrits par vous et visibles des autres joueurs de la partie : n'y mettez pas
            votre vrai nom ni d'information personnelle (adresse, téléphone…).
          </p>
        </section>

        <section>
          <h3>Stockage dans votre navigateur (pas de cookie)</h3>
          <p>
            SpaceJo n'utilise aucun cookie. Il garde dans le stockage de votre navigateur, uniquement sur votre
            appareil, ce qui lui est strictement nécessaire : la partie ouverte dans cet onglet et votre jeton de
            joueur (effacés à la fermeture de l'onglet), la liste des {MAX_KNOWN_SESSIONS} dernières parties rejointes
            sur cet appareil avec leur jeton, pour pouvoir les reprendre, le thème clair ou sombre choisi, et pendant {GITHUB_CACHE_MINUTES} minutes la
            dernière version publiée sur GitHub. Ces informations sont nécessaires au service que vous demandez : elles
            ne demandent donc pas votre consentement (loi Informatique et Libertés, art. 82). Vous pouvez les effacer
            depuis les réglages de votre navigateur ; vous perdez alors seulement l'accès à vos parties en cours.
          </p>
        </section>

        <section>
          <h3>Qui reçoit ces données</h3>
          <p>
            Personne d'autre que les joueurs de votre partie et les prestataires techniques qui hébergent le service,
            pour le compte de l'éditeur :
          </p>
          <ul>
            <li>
              Render (serveur, à Francfort, Allemagne ; société américaine) :{" "}
              <External href="https://render.com/privacy">politique de Render</External> ;
            </li>
            <li>
              Cloudflare (réseau placé devant le serveur, qui voit votre adresse IP ; société américaine) :{" "}
              <External href="https://www.cloudflare.com/privacypolicy/">politique de Cloudflare</External> ;
            </li>
            <li>
              Supabase (base de données, en Irlande ; société basée à Singapour) :{" "}
              <External href="https://supabase.com/privacy">politique de Supabase</External>.
            </li>
          </ul>
          <p>
            Les données sont stockées dans l'Union européenne. Ces sociétés, établies hors de l'Union, peuvent
            toutefois y accéder pour leur service ; elles encadrent ces transferts par les garanties prévues par le
            RGPD (clauses contractuelles types de la Commission européenne, ou cadre de protection des données
            UE–États-Unis pour les sociétés certifiées), décrites dans leurs politiques. Elles peuvent aussi garder votre
            adresse IP dans leurs propres journaux techniques, selon leurs règles.
          </p>
          <p>
            Votre navigateur contacte aussi directement Google Fonts (pour la police Inter) et l'API publique de GitHub
            (pour la version affichée sur l'accueil) : ces services reçoivent votre adresse IP (
            <External href="https://policies.google.com/privacy">politique de Google</External>,{" "}
            <External href="https://docs.github.com/fr/site-policy/privacy-policies/github-general-privacy-statement">
              politique de GitHub
            </External>
            ).
          </p>
        </section>

        <section>
          <h3>Sécurité</h3>
          <p>
            Connexion chiffrée (HTTPS). Jeton secret tiré au hasard (256 bits) et seulement haché côté serveur. Code de
            partie tiré au hasard. Les cartes cachées ne quittent jamais le serveur. Nombre de requêtes limité par
            adresse IP.
          </p>
        </section>

        <section>
          <h3>Vos droits</h3>
          <p>
            Vous pouvez demander l'accès à vos données, leur rectification, leur effacement, la limitation de leur
            traitement ou vous y opposer (RGPD, art. 15 à 21), en écrivant à {mail} avec le code de la partie et le nom
            utilisé. Réponse sous un mois au plus. Comme tout est supprimé {GAME_KEPT_WHEN_IDLE_HOURS} heures après la
            dernière action de la partie, il est possible qu'il n'y ait déjà plus rien à vous communiquer. Pour quitter
            une partie tout de suite, utilisez « Quitter » : au salon, votre nom est retiré ; pendant la partie, il
            reste affiché comme « a abandonné » jusqu'à la suppression de la partie.
          </p>
          <p>
            Si vous estimez que vos droits ne sont pas respectés, vous pouvez saisir la CNIL (
            <External href="https://www.cnil.fr/fr/plaintes">cnil.fr/fr/plaintes</External>).
          </p>
        </section>

        <section>
          <h3>Ce que SpaceJo ne fait pas</h3>
          <p>
            Ni publicité, ni mesure d'audience, ni profilage, ni décision automatisée vous concernant, ni revente de
            données. Aucun âge minimal n'est demandé, puisqu'aucune donnée d'identité n'est collectée.
          </p>
        </section>

        <section>
          <h3>Évolutions</h3>
          <p>
            Si SpaceJo enregistre un jour d'autres données (par exemple un classement ou des comptes), cette page sera
            mise à jour avant, avec la date ci-dessus.
          </p>
        </section>
      </div>
    </WizardFrame>
  );
}
