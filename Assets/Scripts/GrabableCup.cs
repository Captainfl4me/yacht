using System;
using System.Collections;
using Unity.Netcode;
using UnityEngine;

[RequireComponent(typeof(Rigidbody))]
public class GrabableCup : InteractiveObject
{
    private Rigidbody rb;
    private float startTime = -1f;
    [SerializeField] private float shakeAmplitude = 1f;
    [SerializeField] private float shakeFrequency = 1f;
    [SerializeField] private Transform aimPoint;
    [SerializeField] private Transform diceSpawner;

    protected override void Awake()
    {
        base.Awake();
        base.interactiveType = InteractiveType.Grab;
        rb = GetComponent<Rigidbody>();
    }

    public override bool CanInteract()
    {
        return true;
    }

    protected override void OnInteract()
    {
        rb.useGravity = false;
        rb.constraints = RigidbodyConstraints.FreezeRotation;
        startTime = Time.time;
        GameManager.instance.resetPlayingDice();
    }

    protected override void OnEndInteraction()
    {
        startTime = -1f;
        StartCoroutine(DiceSpawnCouroutine());
    }

    IEnumerator DiceSpawnCouroutine()
    {
        rb.isKinematic = true;
        yield return new WaitForEndOfFrame();
        rb.isKinematic = false;
        Quaternion startingRotation = transform.rotation;
        Quaternion targetRotation = Quaternion.LookRotation(aimPoint.position - transform.position);
        float timePassed = 0.0f; //Time passed since the start of the linear interpolation. Starting at 0, it increases until it reaches 1. All values are rendered.

        while (timePassed < 1.0f) //While the time passes is less than 1 (the maximum of a linear interpolation)
        {
            timePassed += Time.deltaTime * 5f;
            transform.rotation = Quaternion.Slerp(startingRotation, targetRotation, timePassed);
            yield return null;
        }


        timePassed = 0.0f;
        startingRotation = transform.rotation;
        targetRotation = startingRotation * Quaternion.AngleAxis(120, Vector3.right);
        while (timePassed < 1.0f) //While the time passes is less than 1 (the maximum of a linear interpolation)
        {
            timePassed += Time.deltaTime * 2f;
            transform.rotation = Quaternion.Slerp(startingRotation, targetRotation, timePassed);
            yield return null;
        }

        for (int i = 0; i < GameManager.instance.getNumberOfDicesToPlay(); i++)
        {
            GameManager.instance.spawnDice(diceSpawner);
            yield return new WaitForSeconds(0.5f);
        }
        yield return new WaitForSeconds(2f);

        rb.useGravity = true;
        rb.constraints = RigidbodyConstraints.None;
    }

    public Rigidbody GetRigidbody()
    {
        return rb;
    }

    private void FixedUpdate()
    {
        if (startTime > 0f)
        {
            Transform target = NetworkManager.Singleton.ConnectedClients[GetIsInteracting().Value].PlayerObject.transform;
            transform.LookAt(target, Vector3.up);
            double rotationValue = shakeAmplitude * Math.Sin(2 * Math.PI * shakeFrequency * (Time.time - startTime));
            transform.rotation = Quaternion.Euler(transform.rotation.eulerAngles.x, transform.rotation.eulerAngles.y, (float)rotationValue);
        }
    }
}
