import { describe, it, expect, beforeEach } from "vitest";
import { GppnClient } from "../src/client.js";
import { GppnIdentity } from "../src/identity.js";
import { ConnectionError, PaymentError } from "../src/errors.js";

describe("GppnClient", () => {
  let client: GppnClient;

  beforeEach(() => {
    client = new GppnClient({ url: "http://localhost:9000" });
  });

  describe("creation", () => {
    it("should create a client with a URL", () => {
      expect(client).toBeDefined();
      expect(client.connected).toBe(false);
      expect(client.identity).toBeUndefined();
    });

    it("should create a client with an existing keypair", async () => {
      const keypair = await GppnIdentity.createIdentity();
      const clientWithKey = new GppnClient({
        url: "http://localhost:9000",
        keypair,
      });

      expect(clientWithKey.identity).toBe(keypair);
    });
  });

  describe("connect/disconnect lifecycle", () => {
    it("should connect successfully", async () => {
      await client.connect();
      expect(client.connected).toBe(true);
    });

    it("should disconnect successfully after connecting", async () => {
      await client.connect();
      await client.disconnect();
      expect(client.connected).toBe(false);
    });

    it("should throw when connecting while already connected", async () => {
      await client.connect();
      await expect(client.connect()).rejects.toThrow(ConnectionError);
      await expect(client.connect()).rejects.toThrow("Already connected");
    });

    it("should throw when disconnecting while not connected", async () => {
      await expect(client.disconnect()).rejects.toThrow(ConnectionError);
      await expect(client.disconnect()).rejects.toThrow("Not connected");
    });
  });

  describe("identity management", () => {
    it("should create a new identity", async () => {
      const identity = await client.createIdentity();
      expect(identity).toBeInstanceOf(GppnIdentity);
      expect(client.identity).toBe(identity);
    });
  });

  describe("payments", () => {
    it("should throw when sending payment while not connected", async () => {
      await client.createIdentity();
      await expect(
        client.sendPayment("recipient", "100", { code: "USD", decimals: 2 })
      ).rejects.toThrow(ConnectionError);
    });

    it("should throw when sending payment without identity", async () => {
      await client.connect();
      await expect(
        client.sendPayment("recipient", "100", { code: "USD", decimals: 2 })
      ).rejects.toThrow(PaymentError);
    });

    it("should send a payment successfully", async () => {
      await client.connect();
      await client.createIdentity();

      const payment = await client.sendPayment(
        "recipient_key",
        "50.00",
        { code: "USD", decimals: 2 },
        "Test payment"
      );

      expect(payment.id).toMatch(/^pay_/);
      expect(payment.recipient).toBe("recipient_key");
      expect(payment.amount.value).toBe("50.00");
      expect(payment.memo).toBe("Test payment");
    });

    it("should retrieve payment status after sending", async () => {
      await client.connect();
      await client.createIdentity();

      const payment = await client.sendPayment(
        "recipient_key",
        "25.00",
        { code: "USD", decimals: 2 }
      );

      const status = await client.getPaymentStatus(payment.id);
      expect(status).toBeDefined();
      expect(status?.id).toBe(payment.id);
    });

    it("should return undefined for unknown payment ID", async () => {
      const status = await client.getPaymentStatus("nonexistent");
      expect(status).toBeUndefined();
    });
  });

  describe("routes", () => {
    it("should throw when finding routes while not connected", async () => {
      await client.createIdentity();
      await expect(
        client.findRoutes("recipient", {
          value: "100",
          currency: { code: "USD", decimals: 2 },
        })
      ).rejects.toThrow(ConnectionError);
    });

    it("should find routes when connected", async () => {
      await client.connect();
      await client.createIdentity();

      const routes = await client.findRoutes("recipient_key", {
        value: "100",
        currency: { code: "USD", decimals: 2 },
      });

      expect(routes).toBeInstanceOf(Array);
      expect(routes.length).toBeGreaterThan(0);
      expect(routes[0].score).toBeGreaterThanOrEqual(routes[routes.length - 1].score);
    });
  });

  describe("trust", () => {
    it("should return a default trust score for unknown peers", async () => {
      const score = await client.getTrustScore("unknown_peer");
      expect(score.peerId).toBe("unknown_peer");
      expect(score.score).toBe(0.5);
    });
  });

  describe("node operations", () => {
    it("should throw when getting peers while not connected", async () => {
      await expect(client.getPeers()).rejects.toThrow(ConnectionError);
    });

    it("should return peers when connected", async () => {
      await client.connect();
      const peers = await client.getPeers();
      expect(peers).toBeInstanceOf(Array);
    });

    it("should throw when getting node status while not connected", async () => {
      await expect(client.getNodeStatus()).rejects.toThrow(ConnectionError);
    });

    it("should return node status when connected", async () => {
      await client.connect();
      const status = await client.getNodeStatus();
      expect(status.connected).toBe(true);
      expect(status.version).toBe("0.1.0");
    });
  });
});
