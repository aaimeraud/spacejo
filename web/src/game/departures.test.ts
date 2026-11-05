import { describe, expect, it } from "vitest";
import type { GameView, PlayerView } from "../api/types";
import { departureNotices, leftBehind } from "./departures";

function player(id: number, name: string, abandoned = false): PlayerView {
  return { id, name, total_score: 0, initial_reveals: 0, missed_turns: 0, eliminated: abandoned, abandoned, grid: [] };
}

function view(phase: GameView["phase"], players: PlayerView[], me = 0): GameView {
  return {
    me,
    host: 0,
    phase,
    round: 1,
    rules: {
      penalty: "DoubleUnlessStrictlyLowest",
      penalty_scope: "RoundTotal",
      first_player_tie: "JoinOrder",
      next_round_starter: "Finisher",
      empty_pile: "ReshuffleDiscard",
      end_tie: "SharedVictory",
      end_score: 100,
      deck: "Full",
    },
    players,
    current_player: null,
    finisher: null,
    held: null,
    draw_pile_count: 0,
    discard_top: null,
    discard_count: 0,
    rounds: [],
    winners: [],
    next_round_votes: [],
    next_round_decider: 0,
  };
}

describe("messages de départ", () => {
  it("annonce à tous un joueur qui abandonne pendant la partie", () => {
    const before = view("Playing", [player(0, "Alice"), player(1, "Bob"), player(2, "Carla")]);
    const after = view("Playing", [player(0, "Alice"), player(1, "Bob"), player(2, "Carla", true)]);
    expect(departureNotices(before, after)).toEqual(["Carla a abandonné la partie."]);
    // Une seule fois : à la vue suivante, rien de nouveau.
    expect(departureNotices(after, after)).toEqual([]);
  });

  it("annonce aussi un abandon qui termine la partie", () => {
    const before = view("Playing", [player(0, "Alice"), player(1, "Bob")]);
    const after = view("GameOver", [player(0, "Alice"), player(1, "Bob", true)]);
    expect(departureNotices(before, after)).toEqual(["Bob a abandonné la partie."]);
  });

  it("annonce un joueur qui quitte le salon, même si les places se décalent", () => {
    const before = view("Lobby", [player(0, "Alice"), player(1, "Bob"), player(2, "Carla")]);
    const after = view("Lobby", [player(0, "Alice"), player(1, "Carla")]);
    expect(departureNotices(before, after)).toEqual(["Bob a quitté le salon."]);
  });

  it("distingue deux joueurs du même nom", () => {
    const before = view("Lobby", [player(0, "Alice"), player(1, "Max"), player(2, "Max")]);
    const after = view("Lobby", [player(0, "Alice"), player(1, "Max")]);
    expect(departureNotices(before, after)).toEqual(["Max a quitté le salon."]);
  });

  it("ne dit rien à la première vue, à l'arrivée d'un joueur, ni pour soi-même", () => {
    const lobby = view("Lobby", [player(0, "Alice")]);
    expect(departureNotices(null, lobby)).toEqual([]);
    expect(departureNotices(lobby, view("Lobby", [player(0, "Alice"), player(1, "Bob")]))).toEqual([]);
    const before = view("Playing", [player(0, "Alice"), player(1, "Bob")], 1);
    const after = view("Playing", [player(0, "Alice"), player(1, "Bob", true)], 1);
    expect(departureNotices(before, after)).toEqual([]);
  });
});

describe("manche suivante sans moi", () => {
  it("me renvoie à l'accueil si la manche a été lancée sans mon vote", () => {
    const before = view("RoundOver", [player(0, "Alice"), player(1, "Bob")], 1);
    const after = view("InitialReveal", [player(0, "Alice"), player(1, "Bob", true)], 1);
    expect(leftBehind(before, after)).toBe(true);
    expect(departureNotices(before, after)).toEqual([]);
  });

  it("annonce aux autres les joueurs qui ne rejoignent pas la manche suivante", () => {
    const before = view("RoundOver", [player(0, "Alice"), player(1, "Bob"), player(2, "Carla")]);
    const after = view("InitialReveal", [player(0, "Alice"), player(1, "Bob"), player(2, "Carla", true)]);
    expect(leftBehind(before, after)).toBe(false);
    expect(departureNotices(before, after)).toEqual(["Carla ne joue pas la manche suivante."]);
  });

  it("ne renvoie personne pendant une manche ni à la première vue", () => {
    const players = [player(0, "Alice"), player(1, "Bob", true)];
    expect(leftBehind(null, view("InitialReveal", players, 1))).toBe(false);
    expect(leftBehind(view("Playing", [player(0, "Alice"), player(1, "Bob")], 1), view("Playing", players, 1))).toBe(false);
  });
});
