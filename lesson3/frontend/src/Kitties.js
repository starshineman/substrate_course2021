import React, {useEffect, useState} from 'react';
import {Form, Grid} from 'semantic-ui-react';

import {useSubstrate} from './substrate-lib';
import {TxButton} from './substrate-lib/components';

import KittyCards from './KittyCards';

export default function Kitties (props) {
  const { api, keyring } = useSubstrate();
  const { accountPair } = props;

  const [kittyCnt, setKittyCnt] = useState(0);
  const [kittyDNAs, setKittyDNAs] = useState([]);
  const [kittyOwners, setKittyOwners] = useState([]);
  const [kittyPrices, setKittyPrices] = useState([]);
  const [kitties, setKitties] = useState([]);
  const [status, setStatus] = useState('');

  const fetchKittyCnt = () => {
    /* TODO: 加代码，从 substrate 端读取数据过来 */
    let unsubscribe;
    api.query.kittiesModule.kittiesCount(count => {
      setKittyCnt(count.toNumber());
    }).then(unsub => {
      unsubscribe = unsub;
    }).catch(console.error);
    return () => unsubscribe && unsubscribe();
  };

  const fetchKitties = () => {
    /* TODO: 加代码，从 substrate 端读取数据过来 */
    let unsubscribe;
    let key_arr = [];
    for (let i=0; i<kittyCnt; i++) {
      key_arr.push(i);
    }
    api.query.kittiesModule.kitties.multi(key_arr, (data) => {
      let new_data = data.map((d, i) => {
        return {
          dna: d,
          id: i,
          is_owner: true
        }
      })
      setKitties(new_data);
    }).then(unsub => {
      unsubscribe = unsub;
    }).catch(console.error);
    return () => unsubscribe && unsubscribe();
  };

  const fetchKittyOwners = () => {
    let unsubscribe;
    let key_arr = [];
    for (let i=0; i<kittyCnt; i++) {
      key_arr.push(i);
    }
    api.query.kittiesModule.kittyOwners.multi(key_arr, (data) => {
      setKittyOwners(data);
    }).then(unsub => {
      unsubscribe = unsub;
    }).catch(console.error);
    return () => unsubscribe && unsubscribe();
  };

  const fetchKittyPrices = () => {
    let unsubscribe;
    let key_arr = [];
    for (let i=0; i<kittyCnt; i++) {
      key_arr.push(i);
    }
    api.query.kittiesModule.kittyPrices.multi(key_arr, (data) => {
      setKittyPrices(data);
    }).then(unsub => {
      unsubscribe = unsub;
    }).catch(console.error);
    return () => unsubscribe && unsubscribe();
  };

  const populateKitties = () => {
    /* TODO: 加代码，从 substrate 端读取数据过来 */
  };

  useEffect(fetchKittyCnt, [api, keyring, setKittyCnt, api.query.kittiesModule]);
  useEffect(fetchKitties, [api, kittyCnt, setKitties, api.query.kittiesModule]);
  useEffect(fetchKittyOwners, [api, kittyCnt, setKittyOwners, api.query.kittiesModule]);
  useEffect(fetchKittyPrices, [api, kittyCnt, setKittyPrices, api.query.kittiesModule]);
  useEffect(populateKitties, [kittyDNAs, kittyOwners]);

  return <Grid.Column width={16}>
    <h1>小毛孩</h1>
    <p>总毛孩个数为: {kittyCnt}</p>
    <Form style={{ margin: '1em 0' }}>
      <Form.Field style={{ textAlign: 'center' }}>
        <TxButton
            accountPair={accountPair} label='创建小毛孩' type='SIGNED-TX' setStatus={setStatus}
            attrs={{
              palletRpc: 'kittiesModule',
              callable: 'create',
              inputParams: [],
              paramFields: []
            }}
        />
      </Form.Field>
    </Form>
    <div style={{ overflowWrap: 'break-word' }}>{status}</div>
    <KittyCards kitties={kitties} kittyOwners={kittyOwners} kittyPrices={kittyPrices} accountPair={accountPair} setStatus={setStatus}/>
  </Grid.Column>;
}
