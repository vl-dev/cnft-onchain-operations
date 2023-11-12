import { Connection } from "@solana/web3.js";
import axios from "axios";

export class WrappedConnection extends Connection {

  rpc_node: string;

  constructor(endpoint: string, commitment: any = "confirmed") {
    super(endpoint, commitment);
    this.rpc_node = endpoint;
  }

  async getAsset(assetId: any): Promise<any> {
    try {
      const response = await axios.post(this.rpc_node, {
        jsonrpc: "2.0",
        method: "getAsset",
        id: "compression-example",
        params: [assetId],
      });
      return response.data.result;
    } catch (error) {
      console.error(error);
    }
  }

  async getAssetProof(assetId: any): Promise<any> {
    try {
      const response = await axios.post(this.rpc_node, {
        jsonrpc: "2.0",
        method: "getAssetProof",
        id: "compression-example",
        params: [assetId],
      });
      return response.data.result;
    } catch (error) {
      console.error(error);
    }
  }

  async getAssetsByOwner(
    assetId: string,
    sortBy: any,
    limit: number,
    page: number,
    before: string,
    after: string
  ): Promise<any> {
    try {
      const response = await axios.post(this.rpc_node, {
        jsonrpc: "2.0",
        method: "getAssetsByOwner",
        id: "rpd-op-123",
        params: [assetId, sortBy, limit, page, before, after],
      });
      //console.log("getAssetsByOwner: " + JSON.stringify(response.data));
      return response.data.result;
    } catch (error) {
      console.error(error);
    }
  }

  async getAssetsByCreator(
    assetId: string,
    sortBy: any,
    limit: number,
    page: number,
    before: string,
    after: string
  ): Promise<any> {
    try {
      const response = await axios.post(this.rpc_node, {
        jsonrpc: "2.0",
        method: "getAssetsByCreator",
        id: "compression-example",
        params: [assetId, true, sortBy, limit, page, null, null],
      });

      return response.data.result;
    } catch (error) {
      console.error(error);
    }
  }

  async getAssetsByAuthority(
    assetId: string,
    sortBy: any,
    limit: number,
    page: number,
    before: string,
    after: string
  ): Promise<any> {
    try {
      const response = await axios.post(this.rpc_node, {
        jsonrpc: "2.0",
        method: "getAssetsByAuthority",
        id: "compression-example",
        params: [assetId, sortBy, limit, page, before, after],
      });
      return response.data.result;
    } catch (error) {
      console.error(error);
    }
  }

  async getAssetsByGroup(
    groupKey: string,
    groupValue: string,
    sortBy: any,
    limit: number,
    page: number,
    before: string,
    after: string
  ): Promise<any> {
    try {
      let events = [];

      const response = await axios.post(this.rpc_node, {
        jsonrpc: "2.0",
        method: "getAssetsByGroup",
        id: "rpd-op-123",
        params: [groupKey, groupValue, sortBy, limit, page, before, after],
      });
      events.push(...response.data.result.items);

      return events;
    } catch (error) {
      console.error(error);
    }
  }

  // This will loop through all pages and return all assets
  async getAllAssetsByGroup(
    groupKey: string,
    groupValue: string,
    sortBy: any,
    limit: number,
    page: number,
    before: string,
    after: string
  ): Promise<any> {
    try {
      let events = [];
      let response = await axios.post(this.rpc_node, {
        jsonrpc: "2.0",
        method: "getAssetsByGroup",
        id: "rpd-op-123",
        params: [groupKey, groupValue, sortBy, limit, page, before, after],
      });

      events.push(...response.data.result.items);

      while (true) {
        console.log("Requested page" + page);

        page += 1;
        response = await axios.post(this.rpc_node, {
          jsonrpc: "2.0",
          method: "getAssetsByGroup",
          id: "rpd-op-123",
          params: [groupKey, groupValue, sortBy, limit, page, before, after],
        });

        events.push(...response.data.result.items);
        if (events.length % 1000 != 0 || response.data.result.items.length == 0) {
          break;
        }
      }

      return events;
    } catch (error) {
      console.error(error);
    }
  }
}

