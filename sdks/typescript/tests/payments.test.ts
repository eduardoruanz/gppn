import { describe, it, expect } from "vitest";
import { PaymentBuilder } from "../src/payments.js";
import { PaymentState } from "../src/types.js";
import { PaymentError } from "../src/errors.js";
import type { Currency } from "../src/types.js";

const USD: Currency = { code: "USD", decimals: 2 };

describe("PaymentBuilder", () => {
  it("should build a valid payment message", () => {
    const payment = new PaymentBuilder()
      .sender("aabbccdd")
      .recipient("11223344")
      .amount("100.00", USD)
      .memo("Test payment")
      .build();

    expect(payment.id).toMatch(/^pay_/);
    expect(payment.sender).toBe("aabbccdd");
    expect(payment.recipient).toBe("11223344");
    expect(payment.amount.value).toBe("100.00");
    expect(payment.amount.currency.code).toBe("USD");
    expect(payment.state).toBe(PaymentState.Created);
    expect(payment.memo).toBe("Test payment");
    expect(payment.createdAt).toBeTruthy();
    expect(payment.updatedAt).toBeTruthy();
  });

  it("should build a payment without a memo", () => {
    const payment = new PaymentBuilder()
      .sender("aabbccdd")
      .recipient("11223344")
      .amount("50.00", USD)
      .build();

    expect(payment.memo).toBeUndefined();
    expect(payment.sender).toBe("aabbccdd");
  });

  it("should return validation errors for missing sender", () => {
    const builder = new PaymentBuilder()
      .recipient("11223344")
      .amount("100.00", USD);

    const errors = builder.validate();
    expect(errors).toContain("sender is required");
  });

  it("should return validation errors for missing recipient", () => {
    const builder = new PaymentBuilder()
      .sender("aabbccdd")
      .amount("100.00", USD);

    const errors = builder.validate();
    expect(errors).toContain("recipient is required");
  });

  it("should return validation errors for missing amount", () => {
    const builder = new PaymentBuilder()
      .sender("aabbccdd")
      .recipient("11223344");

    const errors = builder.validate();
    expect(errors).toContain("amount is required");
  });

  it("should return validation error for zero amount", () => {
    const builder = new PaymentBuilder()
      .sender("aabbccdd")
      .recipient("11223344")
      .amount("0", USD);

    const errors = builder.validate();
    expect(errors).toContain("amount must be a positive number");
  });

  it("should return validation error for negative amount", () => {
    const builder = new PaymentBuilder()
      .sender("aabbccdd")
      .recipient("11223344")
      .amount("-10", USD);

    const errors = builder.validate();
    expect(errors).toContain("amount must be a positive number");
  });

  it("should return validation error when sender equals recipient", () => {
    const builder = new PaymentBuilder()
      .sender("aabbccdd")
      .recipient("aabbccdd")
      .amount("100.00", USD);

    const errors = builder.validate();
    expect(errors).toContain("sender and recipient must be different");
  });

  it("should throw PaymentError when building with missing fields", () => {
    const builder = new PaymentBuilder();

    expect(() => builder.build()).toThrow(PaymentError);
    expect(() => builder.build()).toThrow("Invalid payment");
  });

  it("should throw PaymentError with all missing field messages", () => {
    const builder = new PaymentBuilder();

    try {
      builder.build();
      expect.unreachable("Should have thrown");
    } catch (err) {
      expect(err).toBeInstanceOf(PaymentError);
      const msg = (err as PaymentError).message;
      expect(msg).toContain("sender is required");
      expect(msg).toContain("recipient is required");
      expect(msg).toContain("amount is required");
    }
  });
});
