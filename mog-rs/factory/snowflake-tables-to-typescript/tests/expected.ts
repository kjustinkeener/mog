export interface WebSession {
  webSessionId: number; // TODO(mog): Snowflake column can exceed JavaScript's safe range of 2^53; carry wide values losslessly.
  firstName: string;
  lastName: string;
  updatedAt: string;
}

export interface CustomerProfile {
  customerProfileId: number; // TODO(mog): Snowflake column can exceed JavaScript's safe range of 2^53; carry wide values losslessly.
  accountCode: string;
  email: string | null;
  isActive: boolean;
  signedUpOn: string;
  notes: string | null;
  balance: number; // TODO(mog): Snowflake fixed-point column; JavaScript cannot hold every scale exactly -- use an exact-arithmetic library.
  lastSeenAt: string | null;
}

export interface TelemetryEvent {
  eventId: number; // TODO(mog): Snowflake column can exceed JavaScript's safe range of 2^53; carry wide values losslessly.
  seqNo: number;
  retryCount: number | null;
  amount: number; // TODO(mog): Snowflake fixed-point column; JavaScript cannot hold every scale exactly -- use an exact-arithmetic library.
  rate: number | null;
  isSampled: boolean;
  eventDate: string;
  ingestedAt: string;
  occurredAt: string;
  syncedAt: string | null;
  sourceName: string;
  rawBody: string | null;
  payload: unknown | null; // TODO(mog): Snowflake semi-structured column; narrow the loose mapping with a type guard or a schema validator.
  tags: unknown[] | null; // TODO(mog): Snowflake list column has no declared element type; set the element type yourself.
  context: unknown | null; // TODO(mog): Snowflake semi-structured column; narrow the loose mapping with a type guard or a schema validator.
  checksum: Uint8Array | null;
}
